use crate::core::config::Config;
use crate::core::logger::targets;
use crate::core::state::APP_STATE;
use crate::features::crawler::Browser;
use crate::features::sites::model::posts::Model;
use crate::features::sites::model::{posts, site};
use crate::features::sites::repository::post_repository::PostRepository;
use crate::features::sites::repository::site_repository::SiteRepository;
use crate::features::sites::utility::normalize_link::normalize_link;
use crate::features::sites::utility::site_error_tracker::register_site_error;
use crate::features::sites::validation::post_form::PostForm;
use futures::FutureExt;
use tokio::task::JoinSet;
use tokio::time::{Duration, timeout};
use tracing::{error, warn};

const DEFAULT_POST_PROCESS_TIMEOUT: Duration = Duration::from_secs(45);
const DEFAULT_BROWSER_START_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_CONCURRENT_JOBS: usize = 20;

#[derive(Clone)]
struct PostProcessingConfig {
    concurrency_limit: usize,
    post_timeout: Duration,
    browser_start_timeout: Duration,
}

impl PostProcessingConfig {
    fn from_config(config: &Config) -> Self {
        // Using Config keeps environment parsing centralized and consistent.
        Self {
            concurrency_limit: config.post_concurrency,
            post_timeout: Duration::from_secs(config.post_timeout_seconds),
            browser_start_timeout: Duration::from_secs(config.browser_start_timeout_seconds),
        }
    }

    fn fallback() -> Self {
        // Fallback guarantees the crawler still runs even if the app state is unavailable.
        Self {
            concurrency_limit: DEFAULT_MAX_CONCURRENT_JOBS,
            post_timeout: DEFAULT_POST_PROCESS_TIMEOUT,
            browser_start_timeout: DEFAULT_BROWSER_START_TIMEOUT,
        }
    }
}

#[derive(Clone)]
struct PostContentJob {
    post: Model,
    site: site::Model,
}

enum JobResult {
    Completed(()),
    TimedOut(i64),
    Panicked(i64),
}

struct PostContentOrchestrator {
    config: PostProcessingConfig,
}

impl PostContentOrchestrator {
    fn new(config: PostProcessingConfig) -> Self {
        Self { config }
    }

    async fn run(&self, jobs: Vec<PostContentJob>) {
        if jobs.is_empty() {
            return;
        }

        let mut join_set: JoinSet<JobResult> = JoinSet::new();
        let mut queue = jobs.into_iter();

        for _ in 0..self.config.concurrency_limit {
            if let Some(job) = queue.next() {
                self.spawn_job(&mut join_set, job);
            }
        }

        while let Some(result) = join_set.join_next().await {
            if let Some(next_job) = queue.next() {
                self.spawn_job(&mut join_set, next_job);
            }

            match result {
                Ok(JobResult::Completed(_)) => {}
                Ok(JobResult::TimedOut(post_id)) => {
                    mark_post_failed(post_id, "processing timed out").await;
                }
                Ok(JobResult::Panicked(post_id)) => {
                    mark_post_failed(post_id, "task panicked").await;
                }
                Err(join_error) => {
                    error!(
                        target: targets::CRAWLER_POST,
                        error = %join_error,
                        "Join error while processing posts"
                    )
                }
            }
        }
    }

    fn spawn_job(&self, join_set: &mut JoinSet<JobResult>, job: PostContentJob) {
        let timeout_duration = self.config.post_timeout;
        let browser_timeout = self.config.browser_start_timeout;

        join_set.spawn(async move {
            let post_id = job.post.id;
            let task = async move {
                match timeout(
                    timeout_duration,
                    process_post(job.post, job.site, browser_timeout),
                )
                .await
                {
                    Ok(()) => JobResult::Completed(()),
                    Err(_) => JobResult::TimedOut(post_id),
                }
            };

            match std::panic::AssertUnwindSafe(task).catch_unwind().await {
                Ok(result) => result,
                Err(_) => JobResult::Panicked(post_id),
            }
        });
    }
}

pub async fn get_post_content() {
    let config = load_post_processing_config();

    let posts = match PostRepository::pending_list().await {
        Ok(list) => list,
        Err(e) => {
            error!(target: targets::CRAWLER_POST, error = %e, "Failed to load sites");
            return;
        }
    };

    let jobs: Vec<PostContentJob> = posts
        .into_iter()
        .map(|(post, site)| PostContentJob { post, site })
        .collect();

    PostContentOrchestrator::new(config).run(jobs).await;
}

fn load_post_processing_config() -> PostProcessingConfig {
    // Reading from APP_STATE ensures we reuse the already-loaded configuration.
    APP_STATE
        .get()
        .map(|state| PostProcessingConfig::from_config(&state.config))
        .unwrap_or_else(PostProcessingConfig::fallback)
}

async fn process_post(post: Model, site: site::Model, browser_timeout: Duration) {
    let url = post.url.as_deref().unwrap_or("");
    let path_title = site.path_title.as_deref().unwrap_or("");
    let path_image = site.path_image.as_deref().unwrap_or("");
    let path_video = site.path_video.as_deref().unwrap_or("");
    let path_content = site.path_content.as_deref().unwrap_or("");

    let (title, image, video, content) = {
        let browser = match timeout(browser_timeout, Browser::new(url, None, None)).await {
            Ok(Ok(b)) => b,
            Ok(Err(e)) => {
                error!(
                    target: targets::CRAWLER_POST,
                    post_id = post.id,
                    error = %e,
                    "Browser failed to start for post"
                );
                mark_post_failed(post.id, "browser initialization failed").await;
                return;
            }
            Err(_) => {
                warn!(
                    target: targets::CRAWLER_POST,
                    post_id = post.id,
                    timeout_ms = browser_timeout.as_millis(),
                    "Browser startup timeout for post"
                );
                mark_post_failed(post.id, "browser initialization timed out").await;
                return;
            }
        };

        if let Some(remove_str) = &site.path_remove {
            let selectors: Vec<String> = remove_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !selectors.is_empty()
                && let Err(e) = browser.remove_elements(selectors).await
            {
                warn!(
                    target: targets::CRAWLER_SITE,
                    site_id = site.id,
                    error = %e,
                    "Failed to remove elements"
                );
            }
        }

        let title = browser
            .get_element_text(path_title)
            .await
            .unwrap_or_default();
        let image = normalize_link(
            &site.url,
            &browser
                .get_attr(path_image, "src")
                .await
                .unwrap_or_default(),
        );
        let video = normalize_link(
            &site.url,
            &browser
                .get_attr(path_video, "src")
                .await
                .unwrap_or_default(),
        );
        let content = browser
            .get_element_html(path_content)
            .await
            .unwrap_or_default();

        (title, image, video, content)
    };

    if title.is_empty() && image.is_empty() && video.is_empty() && content.is_empty() {
        mark_post_failed(post.id, "no content extracted").await;
        block(&site).await;
        return;
    }

    if let Err(e) = PostRepository::update(
        post.id,
        PostForm {
            title: Some(title),
            body: Some(content),
            image: Some(image),
            video: Some(video),
            status: posts::PostStatus::COMPLETED,
        },
    )
    .await
    {
        mark_post_failed(post.id, "database update failed").await;
        error!(
            target: targets::CRAWLER_POST,
            post_id = post.id,
            error = %e,
            "Failed to update post"
        );
    }
}

async fn mark_post_failed(post_id: i64, reason: &str) {
    // consistent logging and persistence keep job failures observable
    error!(
        target: targets::CRAWLER_POST,
        post_id,
        reason,
        "Post failed"
    );
    if let Err(db_err) = PostRepository::update_failed(post_id).await {
        error!(
            target: targets::CRAWLER_POST,
            post_id,
            error = %db_err,
            "Failed to mark post as failed"
        );
    }
}

async fn block(site: &site::Model) {
    let count = register_site_error(site.id).await;
    if count >= 5 {
        warn!(
            target: targets::CRAWLER_SITE,
            site_id = site.id,
            error_count = count,
            "Site reached error threshold, disabling"
        );
        if let Err(e) = SiteRepository::disable(site.id).await {
            error!(
                target: targets::CRAWLER_SITE,
                site_id = site.id,
                error = %e,
                "Failed to disable site"
            );
        }
    }
}
