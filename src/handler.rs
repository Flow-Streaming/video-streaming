use std::sync::Arc;

use anyhow::Result;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
};
use reqwest::{Client, StatusCode};
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::VIDEO_BUCKET,
    models::{AppState, CreateShowResponse, Show, VideoUploadResponse},
    supabase,
    video_processor::VideoProcessor,
};

pub async fn upload_video(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<VideoUploadResponse>, (StatusCode, String)> {
    let mut file_name: String = String::default();
    let mut content_type: String = String::default();
    let mut content: Bytes = Bytes::default();

    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(name) = field.name() {
            if name == "file" {
                file_name = field.file_name().unwrap_or("video.mp4").to_string();
                content_type = field.content_type().unwrap_or("video/mp4").to_string();
                content = field.bytes().await.unwrap_or_default();
            }
        }
    }

    if !content_type.starts_with("video/") {
        return Err((StatusCode::BAD_REQUEST, "Invalid content type".to_string()));
    }

    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty content".to_string()));
    }

    // Process the video with FFmpeg
    let (processed_filename, thumbnail_filename) =
        VideoProcessor::process_video(&content, &file_name).await?;

    // Create Supabase service
    let supabase = supabase::SupabaseService::new(state.clone());

    // Get the processed video and thumbnail data
    let processed_video_path = format!("processed_video.mp4");
    let thumbnail_path = format!("thumbnail.jpg");

    let processed_video_data = tokio::fs::read(&processed_video_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read processed video: {}", e),
        )
    })?;

    let thumbnail_data = tokio::fs::read(&thumbnail_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read thumbnail: {}", e),
        )
    })?;

    // Upload both files to Supabase
    supabase
        .upload_file_with_content_type(&processed_filename, processed_video_data, "video/mp4")
        .await?;
    supabase
        .upload_file_with_content_type(&thumbnail_filename, thumbnail_data, "image/jpeg")
        .await?;

    // Get the public URLs
    let video_url = supabase.get_public_url(&processed_filename);
    let thumbnail_url = supabase.get_public_url(&thumbnail_filename);

    // Generate a unique ID for this video
    let video_id = Uuid::new_v4().to_string();

    // Return the response
    Ok(Json(VideoUploadResponse {
        id: video_id,
        video_url,
        thumbnail_url: Some(thumbnail_url),
    }))
}

pub async fn create_show(
    State(state): State<Arc<AppState>>,
    Json(show): Json<Show>,
) -> Result<Json<CreateShowResponse>, (StatusCode, String)> {
    // Validate genre
    let valid_genres = vec!["Revenge", "Billionare", "Asian", "Romance"];
    if !valid_genres.contains(&show.genre.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid genre. Must be one of: {}", valid_genres.join(", ")),
        ));
    }

    // Create Supabase service
    let supabase = supabase::SupabaseService::new(state);

    // Prepare show data
    let show_data = json!({
        "title": show.title,
        "description": show.description,
        "release_date": show.release_date,
        "thumbnail_url": show.thumbnail_url,
        "episode_count": show.episode_count,
        "genre": show.genre,
        "rating": show.rating,
        "status": show.status,
    });

    // Insert into database
    let url = format!("{}/rest/v1/shows", supabase.state.supabase_url);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("apikey", &supabase.state.supabase_api_key)
        .header(
            "Authorization",
            format!("Bearer {}", &supabase.state.supabase_api_key),
        )
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&show_data)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !response.status().is_success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to insert show: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ),
        ));
    }

    // Parse response to get the created show's ID
    let created_show: Vec<serde_json::Value> = response.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse response: {}", e),
        )
    })?;

    if created_show.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "No show data returned after creation".to_string(),
        ));
    }

    let show_id = created_show[0]["id"]
        .as_str()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get show ID from response".to_string(),
        ))?
        .to_string();

    Ok(Json(CreateShowResponse {
        id: show_id,
        title: show.title,
    }))
}
