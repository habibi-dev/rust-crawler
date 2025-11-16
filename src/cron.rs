use crate::core::state::AppState;
use crate::features::sites::cron::{PostCleanupCron, SiteCron};

pub struct Cron;

impl Cron {
    pub async fn start(app_state: AppState) {
        SiteCron::run(app_state.clone()).await.start();
        PostCleanupCron::run(app_state).await.start();
    }
}
