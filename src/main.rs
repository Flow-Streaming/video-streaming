mod config;
mod handlers;
mod models;
mod services;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use handlers::videos;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load configuration
    let state = config::load_config();

    // CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the application router
    let app = Router::new()
        .route(
            "/videos",
            get(videos::list_videos).post(videos::create_video),
        )
        .route(
            "/videos/{id}",
            get(videos::get_video).delete(videos::delete_video),
        )
        .route("/videos/{id}/like", post(videos::like_video))
        .route("/videos/{id}/upload", post(videos::upload_video))
        .route("/videos/{id}/stream", get(videos::stream_video))
        .layer(cors)
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
