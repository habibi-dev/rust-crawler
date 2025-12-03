use crate::core::config::Config;
use crate::core::http::start_http;
use crate::core::logger::{LoggingGuard, targets};
use crate::core::state::{APP_STATE, State};
use crate::cron::Cron;
use crate::routes::Routes;
use anyhow::Context;
use tracing::info;

pub async fn app() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load();

    let _logging_guard =
        LoggingGuard::initialize(&config.log_directory, Some(config.log_retention_days))
            .context("Failed to initialize logger")?;
    info!(
        target: targets::SYSTEM,
        "Logger initialized with daily rotation and retention policy"
    );
    info!(target: targets::SYSTEM, "Configuration loaded and environment prepared");

    // Setup database connection
    let db = Config::setup_database().await?;
    info!(target: targets::SYSTEM, "Database connection established");

    // Initialize application state
    State::init(db, config.clone());
    let state = APP_STATE
        .get()
        .cloned()
        .context("Application state not initialized")?;

    // Start background jobs
    let _ = Cron::start(state.clone()).await;
    info!(target: targets::SYSTEM, "Background cron jobs scheduled");

    // Setup routes and middleware
    let routes = Routes::generate(state);

    // Start the HTTP server
    start_http(routes, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start HTTP server: {}", e))?;

    Ok(())
}
