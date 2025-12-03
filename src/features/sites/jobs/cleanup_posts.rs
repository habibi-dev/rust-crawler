use crate::core::logger::targets;
use crate::features::sites::repository::post_repository::PostRepository;
use crate::utility::state::app_state;
use tracing::{error, info, warn};

pub async fn cleanup_old_posts() {
    let state = app_state();
    let keep_latest = state.config.post_keep_latest;

    if keep_latest == 0 {
        warn!(
            target: targets::SYSTEM,
            "[cron:cleanup_old_posts] skipped because POST_KEEP_LATEST is 0"
        );
        return;
    }

    match PostRepository::cleanup_old_posts(keep_latest).await {
        Ok(deleted) => {
            if deleted > 0 {
                info!(
                    target: targets::SYSTEM,
                    deleted,
                    keep_latest,
                    "[cron:cleanup_old_posts] removed old posts"
                );
            }
        }
        Err(err) => {
            error!(target: targets::SYSTEM, error = %err, "[cron:cleanup_old_posts] failed")
        }
    }
}
