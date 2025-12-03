use crate::core::logger::targets;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::error;

// Type aliases for async job function
type CronFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
type CronFn = Arc<dyn Fn() -> CronFuture + Send + Sync>;

pub struct CronDefinition {
    pub name: &'static str,
    pub interval: Duration,
    pub tasks: Vec<CronFn>,
}

pub struct CronManager {
    pub definitions: Vec<CronDefinition>,
}

impl CronManager {
    pub fn new(definitions: Vec<CronDefinition>) -> Self {
        Self { definitions }
    }

    pub fn start(self) {
        for def in self.definitions {
            tokio::spawn(async move {
                let mut ticker = interval(def.interval);

                loop {
                    ticker.tick().await;

                    for task in &def.tasks {
                        let fut = task.clone()();
                        let name = def.name;

                        tokio::spawn(async move {
                            // Async cron job execution with error visibility
                            if let Err(e) = async {
                                fut.await;
                                Ok::<(), anyhow::Error>(())
                            }
                            .await
                            {
                                error!(target: targets::SYSTEM, "[cron:{name}] failed: {e}");
                            }
                        });
                    }
                }
            });
        }
    }
}

pub fn boxed<F, Fut>(f: F) -> CronFn
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Arc::new(move || {
        let fut = f();
        Box::pin(fut) as CronFuture
    })
}
