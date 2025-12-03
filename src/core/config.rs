use crate::seed::run_all;
use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;
use std::{env, fs};

#[derive(Clone)]
pub struct Config {
    pub domain: String,
    pub final_domain: String,
    pub host: String,
    pub https: bool,
    pub port: u16,
    pub database_url: String,
    pub hmac: String,
    pub max_retry_post: u8,
    pub post_check_interval_minutes: u32,
    pub post_keep_latest: u64,
    pub post_concurrency: usize,
    pub post_timeout_seconds: u64,
    pub browser_start_timeout_seconds: u64,
}

impl Config {
    pub fn create_env_file() {
        let env_path = ".env";
        const ENV_SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.example"));

        if !Path::new(env_path).exists() {
            match fs::write(env_path, ENV_SAMPLE) {
                Ok(_) => {
                    println!(".env file created from sample");

                    #[cfg(unix)]
                    {
                        if let Ok(meta) = fs::metadata(env_path) {
                            let mut perms = meta.permissions();
                            perms.set_mode(0o600);
                            if let Err(e) = fs::set_permissions(env_path, perms) {
                                eprintln!("Failed to set permissions: {}", e);
                            }
                        }
                    }
                }
                Err(err) => eprintln!("Failed to create .env file: {}", err),
            }
        }

        if let Err(e) = dotenvy::dotenv() {
            eprintln!("Error loading .env: {}", e);
        }
    }

    pub async fn setup_database() -> Result<sea_orm::DatabaseConnection> {
        let db_url = Self::database_url();
        let connect_options = Self::build_connect_options(&db_url);

        let db = Database::connect(connect_options)
            .await
            .context("Failed to connect to database")?;

        Migrator::up(&db, None)
            .await
            .context("Database migration failed")?;

        run_all(&db).await?;

        Ok(db)
    }

    fn build_connect_options(database_url: &str) -> ConnectOptions {
        let mut options = ConnectOptions::new(database_url.to_owned());

        options
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(15))
            .idle_timeout(Duration::from_secs(300))
            .sqlx_logging(false);

        options
    }

    pub fn load() -> Self {
        Self::create_env_file();
        Self {
            host: Self::app_host(),
            https: Self::app_https(),
            port: Self::app_port(),
            database_url: Self::database_url(),
            domain: Self::domain(),
            final_domain: Self::final_domain(),
            hmac: Self::app_hmac(),
            max_retry_post: Self::app_max_retry_post(),
            post_check_interval_minutes: Self::app_post_check_interval_minutes(),
            post_keep_latest: Self::app_post_keep_latest(),
            post_concurrency: Self::post_processing_concurrency(),
            post_timeout_seconds: Self::post_processing_timeout_seconds(),
            browser_start_timeout_seconds: Self::browser_start_timeout_seconds(),
        }
    }

    fn app_https() -> bool {
        env::var("APP_HTTPS")
            .unwrap_or_else(|_| "false".into())
            .parse()
            .unwrap_or(false)
    }

    fn app_hmac() -> String {
        env::var("HMAC_KEY").unwrap_or_default()
    }

    fn app_max_retry_post() -> u8 {
        env::var("MAX_RETRY_POST")
            .unwrap_or(String::from("3"))
            .parse()
            .unwrap_or(3)
    }

    fn app_post_check_interval_minutes() -> u32 {
        env::var("POST_CHECK_INTERVAL_MINUTES")
            .unwrap_or(String::from("15"))
            .parse()
            .unwrap_or(15)
    }

    fn app_post_keep_latest() -> u64 {
        env::var("POST_KEEP_LATEST")
            .unwrap_or_else(|_| String::from("1000"))
            .parse()
            .unwrap_or(1000)
    }

    fn app_host() -> String {
        env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into())
    }

    fn final_domain() -> String {
        env::var("APP_FINAL_DOMAIN").unwrap_or_else(|_| "localhost".into())
    }

    fn domain() -> String {
        env::var("APP_DOMAIN").unwrap_or_else(|_| "localhost".into())
    }

    fn database_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.db?mode=rwc".into())
    }

    fn app_port() -> u16 {
        env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .unwrap_or(8080)
    }

    fn post_processing_concurrency() -> usize {
        env::var("CRAWLER_POST_CONCURRENCY")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(10)
    }

    fn post_processing_timeout_seconds() -> u64 {
        env::var("CRAWLER_POST_TIMEOUT")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(15)
    }

    fn browser_start_timeout_seconds() -> u64 {
        env::var("CRAWLER_BROWSER_TIMEOUT")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(25)
    }
}
