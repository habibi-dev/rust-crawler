use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing_subscriber::filter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

pub mod targets {
    pub const REQUEST: &str = "request";
    pub const CRAWLER_SITE: &str = "crawler_site";
    pub const CRAWLER_POST: &str = "crawler_post";
    pub const SYSTEM: &str = "system";
}

const DEFAULT_RETENTION_DAYS: u64 = 3;

pub struct LoggingGuard {
    _guards: Arc<Vec<WorkerGuard>>,
    retention_days: u64,
    base_dir: PathBuf,
}

impl LoggingGuard {
    pub fn initialize(base_dir: impl AsRef<Path>, retention_days: Option<u64>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        let retention_days = retention_days.unwrap_or(DEFAULT_RETENTION_DAYS);

        let request_dir = base_dir.join("requests");
        let crawler_site_dir = base_dir.join("crawler_sites");
        let crawler_post_dir = base_dir.join("crawler_posts");
        let system_dir = base_dir.join("system");

        Self::create_directory(&request_dir)?;
        Self::create_directory(&crawler_site_dir)?;
        Self::create_directory(&crawler_post_dir)?;
        Self::create_directory(&system_dir)?;

        let mut guards: Vec<WorkerGuard> = Vec::new();

        let request_layer = Self::build_layer(targets::REQUEST, &request_dir, &mut guards)?;
        let crawler_site_layer =
            Self::build_layer(targets::CRAWLER_SITE, &crawler_site_dir, &mut guards)?;
        let crawler_post_layer =
            Self::build_layer(targets::CRAWLER_POST, &crawler_post_dir, &mut guards)?;
        let system_layer = Self::build_layer(targets::SYSTEM, &system_dir, &mut guards)?;

        let layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> = vec![
            request_layer,
            crawler_site_layer,
            crawler_post_layer,
            system_layer,
        ];

        let subscriber = Registry::default().with(layers);

        subscriber
            .try_init()
            .map_err(|e| anyhow::anyhow!("Logger initialization failed: {}", e))?;

        let guard = Self {
            retention_days,
            base_dir,
            _guards: Arc::new(guards),
        };

        guard.cleanup_old_logs()?;

        Ok(guard)
    }

    fn cleanup_old_logs(&self) -> Result<()> {
        let retention = Duration::from_secs(self.retention_days * 24 * 60 * 60);
        self.cleanup_directory(&self.base_dir.join("requests"), retention)?;
        self.cleanup_directory(&self.base_dir.join("crawler_sites"), retention)?;
        self.cleanup_directory(&self.base_dir.join("crawler_posts"), retention)?;
        self.cleanup_directory(&self.base_dir.join("system"), retention)?;
        Ok(())
    }

    fn cleanup_directory(&self, directory: &Path, retention: Duration) -> Result<()> {
        if !directory.exists() {
            return Ok(());
        }

        let now = SystemTime::now();

        for entry in fs::read_dir(directory).context("Failed to read log directory")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let metadata = entry.metadata().context("Failed to read log metadata")?;
            if let Ok(modified) = metadata.modified()
                && let Ok(elapsed) = now.duration_since(modified)
                && elapsed > retention
            {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove old log {:?}", path))?;
            }
        }

        Ok(())
    }

    fn create_directory(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create log directory {:?}", path))?;
        }
        Ok(())
    }

    fn build_layer(
        target: &'static str,
        directory: &Path,
        guards: &mut Vec<WorkerGuard>,
    ) -> Result<Box<dyn Layer<Registry> + Send + Sync + 'static>> {
        let appender = DailyLogWriter::new(directory, target)?;
        let (writer, guard) = non_blocking::NonBlockingBuilder::default()
            .lossy(false)
            .finish(appender);

        guards.push(guard);

        let layer = fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(writer)
            .with_filter(filter::filter_fn(move |metadata| {
                metadata.target() == target
            }));

        Ok(layer.boxed())
    }
}

// Holds current file and date for rotation.
struct DailyLogState {
    current_date: NaiveDate,
    file: File,
}

// Writer that keeps an "active" file (target.log) for today and archives
// previous day's file as target-YYYY-MM-DD.log on date change.
struct DailyLogWriter {
    directory: PathBuf,
    file_stem: String,
    state: Mutex<DailyLogState>,
}

impl DailyLogWriter {
    fn new(directory: &Path, file_stem: &str) -> Result<Self> {
        if !directory.exists() {
            fs::create_dir_all(directory).context("Failed to create log directory")?;
        }

        let state = Self::prepare_state(directory, file_stem)?;

        Ok(Self {
            directory: directory.to_path_buf(),
            file_stem: file_stem.to_string(),
            state: Mutex::new(state),
        })
    }

    fn prepare_state(directory: &Path, file_stem: &str) -> Result<DailyLogState> {
        let today = Local::now().date_naive();
        let active_path = Self::active_path(directory, file_stem);

        if let Ok(metadata) = fs::metadata(&active_path)
            && let Ok(modified) = metadata.modified()
        {
            let modified_date = DateTime::<Local>::from(modified).date_naive();
            if modified_date != today {
                let archived_path = Self::archived_path(directory, file_stem, modified_date);
                if let Err(err) = fs::rename(&active_path, &archived_path) {
                    return Err(anyhow::anyhow!(
                        "Failed to archive outdated log file {:?}: {}",
                        active_path,
                        err
                    ));
                }
            }
        }

        let file = Self::open_active_file(&active_path)?;

        Ok(DailyLogState {
            current_date: today,
            file,
        })
    }

    fn open_active_file(path: &Path) -> Result<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("Failed to open log file at {:?}", path))
    }

    fn active_path(directory: &Path, file_stem: &str) -> PathBuf {
        directory.join(format!("{}.log", file_stem))
    }

    fn archived_path(directory: &Path, file_stem: &str, date: NaiveDate) -> PathBuf {
        directory.join(format!("{}-{}.log", file_stem, date.format("%Y-%m-%d")))
    }

    fn rotate_if_needed(&self, state: &mut DailyLogState) -> io::Result<()> {
        let today = Local::now().date_naive();
        if state.current_date == today {
            return Ok(());
        }

        let active_path = Self::active_path(&self.directory, &self.file_stem);
        let archived_path =
            Self::archived_path(&self.directory, &self.file_stem, state.current_date);

        if active_path.exists() {
            fs::rename(&active_path, &archived_path)?;
        }

        state.file =
            Self::open_active_file(&active_path).map_err(|e| io::Error::other(e.to_string()))?;
        state.current_date = today;

        Ok(())
    }
}

impl Write for DailyLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("Logger state poisoned"))?;

        self.rotate_if_needed(&mut state)?;
        state.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("Logger state poisoned"))?;

        state.file.flush()
    }
}
