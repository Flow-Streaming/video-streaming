use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub video_url: String,
    pub thumbnail_url: Option<String>,
    pub user_id: String,
    pub created_at: String,
    pub likes: i32,
    pub views: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVideoRequest {
    pub title: String,
    pub description: Option<String>,
    pub user_id: String, // Added user_id
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVideoResponse {
    pub id: String,
    pub title: String,
    pub upload_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub stream_url: String,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    pub likes: i32,
    pub views: i32,
    pub user_id: String,
}

#[derive(Clone)]
pub struct AppState {
    pub supabase_url: String,
    pub supabase_api_key: String,
    pub supabase_bucket: String,
}
