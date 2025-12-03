use crate::core::config::Config;
use crate::utility::url::url;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_http(app: Router, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("{}:{}", config.host, config.port);
    let url = url("/");

    info!(target: "system", "HTTP listener bound (no TLS) on {}", address);
    info!(target: "system", "Domain configured: {}", url);

    let listener = TcpListener::bind(&address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
