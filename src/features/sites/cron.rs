use crate::core::cron_manager::{CronDefinition, CronManager, boxed};
use crate::core::state::AppState;
use crate::features::sites::jobs::check_new_post::check_new_post;
use crate::features::sites::jobs::get_post_content::get_post_content;
use std::time::Duration;

pub struct SiteCron;
impl SiteCron {
    pub async fn run(app_state: AppState) -> CronManager {
        let m = app_state.config.post_check_interval_minutes;
        let jon_time = Duration::from_secs((m * 60) as u64);
        let job = CronDefinition {
            name: "fetch_new_posts",
            interval: jon_time,
            tasks: vec![
                boxed(|| async { check_new_post().await }),
                boxed(|| async { get_post_content().await }),
            ],
        };
        CronManager::new(vec![job])
    }
}
