use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::error;

pub async fn run_server() -> Result<(), anyhow::Error> {
    let server = Router::new().nest_service("/", ServeDir::new("frontend/assets"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, server).await.map_err(|e| {
        error!("Server failed to run: {}", e);
        e
    })?;
    Ok(())
}
