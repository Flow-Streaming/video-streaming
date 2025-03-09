use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use serde_json::json;
use tokio::{fs::File, io::AsyncWriteExt};
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

            VideoMetadata {
                id: video.id,
                title: video.title,
                description: video.description,
                stream_url,
                thumbnail_url: video.thumbnail_url,
                created_at: video.created_at,
                likes: video.likes,
                views: video.views,
                user_id: video.user_id,
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

    // Increment views
    supabase
        .call_rpc("increment_views", json!({ "video_id": id }))
        .await?;

    let video: Video = supabase.query_single("videos", "id", &id).await?;

    let stream_url = format!("/videos/{}/stream", video.id);

    let metadata = VideoMetadata {
        id: video.id,
        title: video.title,
        description: video.description,
        stream_url,
        thumbnail_url: video.thumbnail_url,
        created_at: video.created_at,
        likes: video.likes,
        views: video.views,
        user_id: video.user_id,
    };

    Ok(Json(metadata))
}

// Create a new video
pub async fn create_video(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVideoRequest>,
) -> Result<Json<CreateVideoResponse>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let file_path = format!("videos/{}.mp4", id);
    let video_url = format!(
        "{}/storage/v1/object/public/{}/{}",
        state.supabase_url, state.supabase_bucket, file_path
    );

    let supabase = SupabaseService::new(state.clone());

    let video = json!({
        "id": id,
        "title": payload.title,
        "description": payload.description,
        "video_url": video_url,
        "user_id": payload.user_id,
        "likes": 0,
        "views": 0
    });

    supabase.insert::<()>("videos", video).await?;

    let upload_url = format!("/videos/{}/upload", id);

    Ok(Json(CreateVideoResponse {
        id,
        title: payload.title,
        upload_url,
    }))
}

// Upload a video file
pub async fn upload_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    // First, let's check if the video exists
    let _: Video = supabase.query_single("videos", "id", &id).await?;

    // Process multipart form
    let temp_path = format!("/tmp/video-{}.mp4", id);
    let mut thumbnail_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "video" {
            let content_type = field.content_type().unwrap_or("").to_string();

            if !content_type.starts_with("video/") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid file type, expected video".to_string(),
                ));
            }

            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Write to temporary file
            let mut file = File::create(&temp_path)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            file.write_all(&data)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            file.flush()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        } else if name == "thumbnail" {
            let content_type = field.content_type().unwrap_or("").to_string();

            if !content_type.starts_with("image/") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid file type for thumbnail, expected image".to_string(),
                ));
            }

            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Upload thumbnail
            let thumb_path = format!("thumbnails/{}.jpg", id);

            supabase
                .upload_file(
                    &state.supabase_bucket,
                    &thumb_path,
                    data.to_vec(),
                    "image/jpeg",
                )
                .await?;

            let thumb_url = supabase.get_public_url(&state.supabase_bucket, &thumb_path);
            thumbnail_path = Some(thumb_url);
        }
    }

    // Upload to Supabase Storage
    let file_path = format!("videos/{}.mp4", id);

    // Read file content
    let file_content = tokio::fs::read(&temp_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Upload to Supabase Storage
    supabase
        .upload_file(
            &state.supabase_bucket,
            &file_path,
            file_content,
            "video/mp4",
        )
        .await?;

    // Update thumbnail URL if available
    if let Some(thumb_url) = thumbnail_path {
        let update = json!({
            "thumbnail_url": thumb_url
        });

        supabase.update("videos", "id", &id, update).await?;
    }

    // Clean up temporary file
    tokio::fs::remove_file(&temp_path).await.ok();

    Ok(StatusCode::OK)
}

// Stream a video
pub async fn stream_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    #[derive(serde::Deserialize)]
    struct VideoUrl {
        video_url: String,
    }

    let video: VideoUrl = supabase.query_single("videos", "id", &id).await?;

    Ok(Json(video.video_url))
}

// Delete a video
pub async fn delete_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    let video: Video = supabase.query_single("videos", "id", &id).await?;

    // Extract filenames from URLs
    let video_path = video.video_url.split('/').last().unwrap_or("");
    let thumbnail_path = video
        .thumbnail_url
        .as_ref()
        .map(|url| url.split('/').last().unwrap_or(""));

    // Delete from database first
    supabase.delete("videos", "id", &id).await?;

    // Delete files from storage if they exist
    if !video_path.is_empty() {
        supabase
            .delete_file(&state.supabase_bucket, &format!("videos/{}", video_path))
            .await
            .ok();
    }

    if let Some(path) = thumbnail_path {
        if !path.is_empty() {
            supabase
                .delete_file(&state.supabase_bucket, &format!("thumbnails/{}", path))
                .await
                .ok();
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// Add like to a video
pub async fn like_video(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let supabase = SupabaseService::new(state.clone());

    // Call the RPC function to toggle like
    supabase
        .call_rpc("toggle_like", json!({ "video_id": id }))
        .await?;

    Ok(StatusCode::OK)
}
