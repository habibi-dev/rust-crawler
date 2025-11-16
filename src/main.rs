use rust_crawler::app::app;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if let Err(e) = app().await {
        eprintln!("Application failed to start: {}", e);
        std::process::exit(1);
    }
}
