use crate::features::sites::repository::post_repository::PostRepository;
use crate::utility::state::app_state;

pub async fn cleanup_old_posts() {
    let state = app_state();
    let keep_latest = state.config.post_keep_latest;

    if keep_latest == 0 {
        eprintln!("[cron:cleanup_old_posts] skipped because POST_KEEP_LATEST is 0");
        return;
    }

    match PostRepository::cleanup_old_posts(keep_latest).await {
        Ok(deleted) => {
            if deleted > 0 {
                println!(
                    "[cron:cleanup_old_posts] deleted {} posts older than the latest {} entries",
                    deleted, keep_latest
                );
            }
        }
        Err(err) => eprintln!("[cron:cleanup_old_posts] failed: {}", err),
    }
}
