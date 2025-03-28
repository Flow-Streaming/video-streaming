use anyhow::Result;
use axum::{
    Router, // routing::{get, post},
    routing::post,
};
use reqwest::Method;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

pub mod config;
pub mod handler;
pub mod models;
pub mod supabase;
pub mod video_processor;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load configuration
    let load_config = config::load_config();
    let state = load_config;
    info!("Configuration loaded successfully");

    // Enhanced CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    info!("Setting up routes");

    // Build the application router
    let app = Router::new()
        .route("/videos", post(handler::upload_video))
        .route("/shows", post(handler::create_show))
        // .route(
        //     "/videos/{id}",
        //     get(videos::get_video).delete(videos::delete_video),
        // )
        // // .route("/videos/{id}/upload", post(videos::upload_video))
        // .route("/videos/{id}/stream", get(videos::stream_video))
        .layer(cors)
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server starting on {}", addr);

    // Use tokio's TcpListener instead of std's
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    // await the serve call instead of using the ? operator
    axum::serve(listener, app).await?;

    Ok(())
}
