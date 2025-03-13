mod config;
mod handlers;
mod models;
mod services;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Method;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use handlers::videos;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with more detailed output
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load configuration
    let state = config::load_config();
    info!("Configuration loaded successfully");

    // Enhanced CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    info!("Setting up routes");

    // Build the application router
    let app = Router::new()
        .route(
            "/videos",
            get(videos::list_videos)
                .post(videos::create_video)
        )
        .route(
            "/videos/{id}",
            get(videos::get_video)
                .delete(videos::delete_video)
        )
        .route("/videos/{id}/upload", post(videos::upload_video))
        .route("/videos/{id}/stream", get(videos::stream_video))
        .layer(cors)
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server starting on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
