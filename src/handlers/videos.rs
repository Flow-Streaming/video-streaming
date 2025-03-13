use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use serde_json::json;
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    models::{AppState, CreateVideoRequest, CreateVideoResponse, Video, VideoMetadata},
    services::supabase::SupabaseService,
};

// List all videos
pub async fn list_videos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<VideoMetadata>>, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    let videos: Vec<Video> = supabase
        .query_all("videos", Some("created_at.desc"))
        .await?;

    let videos_metadata: Vec<VideoMetadata> = videos
        .into_iter()
        .map(|video| {
            let stream_url = format!("/videos/{}/stream", video.id);
            let thumbnail_url = video
                .thumbnail_url
                .map(|path| supabase.get_public_url(&state.supabase_bucket, &path));

            VideoMetadata {
                id: video.id,
                title: video.title,
                description: video.description,
                stream_url,
                thumbnail_url,
                created_at: video.created_at,
            }
        })
        .collect();

    Ok(Json(videos_metadata))
}

// Get a specific video
pub async fn get_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<VideoMetadata>, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    let video: Video = supabase.query_single("videos", "id", &id).await?;

    let stream_url = format!("/videos/{}/stream", video.id);
    let thumbnail_url = video
        .thumbnail_url
        .map(|path| supabase.get_public_url(&state.supabase_bucket, &path));

    let metadata = VideoMetadata {
        id: video.id,
        title: video.title,
        description: video.description,
        stream_url,
        thumbnail_url,
        created_at: video.created_at,
    };

    Ok(Json(metadata))
}

// Create a new video - FIXED
pub async fn create_video(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVideoRequest>,
) -> Result<Json<CreateVideoResponse>, (StatusCode, String)> {
    // Log the incoming request payload
    info!("Create video request: {:?}", payload);

    // Generate a unique ID for the video
    let id = Uuid::new_v4().to_string();

    // Set up the URLs based on how your system works
    let video_url = format!("/videos/{}/stream", id);
    let thumbnail_url = "None"; // Can be updated later

    let supabase = SupabaseService::new(state.clone());

    // Prepare data for insertion - MATCH EXACTLY THE DATABASE SCHEMA
    let video = json!({
        "id": id,
        "title": payload.title,
        "description": payload.description,
        "video_url": video_url,
        "thumbnail_url": thumbnail_url
        // Don't include created_at - it has a default value
        // Don't include likes and views - they have default values
    });

    // Insert into database
    info!(
        "Attempting to insert video with data: {}",
        video.to_string()
    );
    match supabase.insert::<()>("videos", video.clone()).await {
        Ok(_) => {
            info!("Video entry created successfully with ID: {}", id);
        }
        Err(e) => {
            error!("Failed to insert video entry: {:?}", e);
            return Err(e);
        }
    }

    // Generate upload URL
    let upload_url = format!("/videos/{}/upload", id);

    // Create the response
    let response = CreateVideoResponse {
        id: id.clone(),
        title: payload.title.clone(),
        upload_url,
    };

    info!("Sending create video response: {:?}", response);
    Ok(Json(response))
}
// Upload a video file
pub async fn upload_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("Upload request received for video ID: {}", id);

    let supabase = SupabaseService::new(state.clone());

    // First, let's check if the video exists
    match supabase.query_single::<Video>("videos", "id", &id).await {
        Ok(_) => info!("Video entry found, proceeding with upload"),
        Err(e) => {
            error!("Video lookup failed: {:?}", e);
            return Err(e);
        }
    }

    // Process multipart form
    let temp_path = format!("/tmp/video-{}.mp4", id);
    let mut thumbnail_path = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart field error: {}", e);
        (StatusCode::BAD_REQUEST, e.to_string())
    })? {
        let name = field.name().unwrap_or("").to_string();
        info!("Processing field: {}", name);

        if name == "video" {
            let content_type = field.content_type().unwrap_or("").to_string();

            if !content_type.starts_with("video/") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid file type, expected video".to_string(),
                ));
            }

            let data = field.bytes().await.map_err(|e| {
                error!("Error reading video bytes: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            info!("Received video data, size: {} bytes", data.len());

            // Write to temporary file
            let mut file = File::create(&temp_path).await.map_err(|e| {
                error!("Error creating temp file: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            file.write_all(&data).await.map_err(|e| {
                error!("Error writing to temp file: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            file.flush().await.map_err(|e| {
                error!("Error flushing temp file: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            info!("Video saved to temporary path: {}", temp_path);
        } else if name == "thumbnail" {
            let content_type = field.content_type().unwrap_or("").to_string();

            if !content_type.starts_with("image/") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid file type for thumbnail, expected image".to_string(),
                ));
            }

            let data = field.bytes().await.map_err(|e| {
                error!("Error reading thumbnail bytes: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

            info!("Received thumbnail data, size: {} bytes", data.len());

            // Upload thumbnail to Supabase
            let thumb_path = format!("thumbnails/{}.jpg", id);
            match supabase
                .upload_file(
                    &state.supabase_bucket,
                    &thumb_path,
                    data.to_vec(),
                    "image/jpeg",
                )
                .await
            {
                Ok(_) => {
                    info!("Thumbnail uploaded successfully");
                    thumbnail_path = Some(thumb_path);
                }
                Err(e) => {
                    error!("Failed to upload thumbnail: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    // Upload to Supabase Storage
    let file_path = format!("videos/{}.mp4", id);

    // Read file content
    let file_content = tokio::fs::read(&temp_path).await.map_err(|e| {
        error!("Error reading temp file: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    info!(
        "Uploading video to Supabase, size: {} bytes",
        file_content.len()
    );

    // Upload to Supabase Storage
    match supabase
        .upload_file(
            &state.supabase_bucket,
            &file_path,
            file_content,
            "video/mp4",
        )
        .await
    {
        Ok(_) => info!("Video file uploaded successfully"),
        Err(e) => {
            error!("Failed to upload video file: {:?}", e);
            return Err(e);
        }
    }

    // Update thumbnail path if available
    if let Some(thumb_path) = thumbnail_path {
        info!("Updating video with thumbnail path: {}", thumb_path);
        let update = json!({
            "thumbnail_path": thumb_path
        });

        match supabase.update("videos", "id", &id, update).await {
            Ok(_) => info!("Video updated with thumbnail path"),
            Err(e) => {
                error!("Failed to update video with thumbnail: {:?}", e);
                return Err(e);
            }
        }
    }

    // Clean up temporary file
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        error!("Failed to clean up temp file: {}", e);
    } else {
        info!("Temporary file cleaned up");
    }

    info!("Upload process completed successfully for video ID: {}", id);
    Ok(StatusCode::OK)
}

// Stream a video
pub async fn stream_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    let video: Video = supabase.query_single("videos", "id", &id).await?;

    let streaming_url = if video.video_url.starts_with("http") {
        video.video_url
    } else {
        // Construct the storage URL
        let file_path = format!("videos/{}.mp4", id);
        supabase.get_public_url(&state.supabase_bucket, &file_path)
    };

    Ok(Json(streaming_url))
}

// Delete a video
pub async fn delete_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    // Get the video to know what files to delete
    let video: Video = supabase.query_single("videos", "id", &id).await?;

    // Delete from database first
    supabase.delete("videos", "id", &id).await?;

    // Then delete from storage - construct the file path
    let file_path = format!("videos/{}.mp4", id);
    supabase
        .delete_file(&state.supabase_bucket, &file_path)
        .await?;

    // Delete thumbnail if exists
    if let Some(thumbnail_url) = video.thumbnail_url {
        // Extract the path from the URL if needed
        if let Some(path) = thumbnail_url.strip_prefix(&format!(
            "{}/storage/v1/object/public/{}/",
            state.supabase_url, state.supabase_bucket
        )) {
            supabase
                .delete_file(&state.supabase_bucket, path)
                .await
                .ok();
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
