use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
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

        // Collect all layers into a Vec; Vec<L> where L: Layer<Registry> itself
        // implements Layer<Registry>, so we only need a single `.with(...)`.
        let layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> = vec![
            request_layer.boxed(),
            crawler_site_layer.boxed(),
            crawler_post_layer.boxed(),
            system_layer.boxed(),
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
    ) -> Result<impl Layer<Registry> + Send + Sync + 'static> {
        let file_name = format!("{}.log", target);
        let appender = RollingFileAppender::new(Rotation::DAILY, directory, file_name);
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

        Ok(layer)
    }
}
