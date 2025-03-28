use std::sync::Arc;

use anyhow::Result;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
};
use reqwest::StatusCode;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    models::{AppState, VideoUploadResponse},
    supabase,
    video_processor::VideoProcessor,
};

pub async fn raw_upload(body: Bytes) -> Result<String, (StatusCode, String)> {
    info!("Received raw upload of {} bytes", body.len());

    // Just acknowledge receipt of the data
    Ok(format!("Successfully received {} bytes", body.len()))
}

pub async fn simple_multipart(mut multipart: Multipart) -> Result<String, (StatusCode, String)> {
    info!("Starting simple multipart test");

    let mut fields_count = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("(no name)");
        let file_name = field.file_name().unwrap_or("(no file name)");
        let content_type = field.content_type().unwrap_or("(no content type)");

        info!(
            "Field: name={}, filename={}, content_type={}",
            name, file_name, content_type
        );

        // Don't read the content for now, just acknowledge the field
        fields_count += 1;
    }

    Ok(format!("Successfully processed {} fields", fields_count))
}

pub async fn upload_video(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<VideoUploadResponse>, (StatusCode, String)> {
    info!("Starting to process multipart upload");

    let mut file_name: String = String::default();
    let mut _content_type: String = String::default();
    let mut content: Bytes = Bytes::default();

    // Handle the next_field result properly
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.name() {
            info!("Processing field: {}", name);
            if name == "file" {
                file_name = field.file_name().unwrap_or("video.mp4").to_string();
                _content_type = field.content_type().unwrap_or("video/mp4").to_string();
                info!("File name: {}, Content-Type: {}", file_name, _content_type);

                // Handle bytes result properly
                content = match field.bytes().await {
                    Ok(bytes) => {
                        info!("Successfully read {} bytes", bytes.len());
                        bytes
                    }
                    Err(err) => {
                        error!("Failed to read file bytes: {}", err);
                        return Err((
                            StatusCode::BAD_REQUEST,
                            format!("Failed to read file bytes: {}", err),
                        ));
                    }
                };
            }
        }
    }

    if content.is_empty() {
        error!("No file content found");
        return Err((StatusCode::BAD_REQUEST, "No file content found".to_string()));
    }

    // Process the video with FFmpeg
    info!("Processing video: {} ({} bytes)", file_name, content.len());
    let (output_path, filename) = VideoProcessor::process_video(&content, &file_name).await?;

    // Create Supabase service
    let supabase = supabase::SupabaseService::new(state.clone()).await?;

    // Get the processed video data
    info!("Trying to read processed video from: {}", &output_path);
    if tokio::fs::try_exists(&output_path).await.unwrap_or(false) {
        info!("File exists at path");
    } else {
        error!("File does not exist at path: {}", &output_path);
    }
    let processed_video_data = tokio::fs::read(&output_path).await.map_err(|e| {
        error!("Failed to read processed video: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read processed video: {}", e),
        )
    })?;

    info!("Uploading processed video to storage");
    // Upload the processed video to Supabase
    supabase
        .upload_file_with_content_type(
            &filename,
            processed_video_data,
            "video/mp4",
            "937f5714-ab0b-471b-a66d-d07e3b68af70",
        )
        .await?;

    // Get the public URLs
    let video_url = supabase.get_public_url(&filename);

    // Generate a unique ID for this video
    let video_id = Uuid::new_v4().to_string();

    info!("Video upload complete. ID: {}", video_id);
    // Return the response
    Ok(Json(VideoUploadResponse {
        id: video_id,
        video_url,
        thumbnail_url: Some("thumbnail_url".to_string()),
    }))
}
