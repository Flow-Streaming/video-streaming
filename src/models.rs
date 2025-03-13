use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub video_url: String,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub likes: i32,
    #[serde(default)]
    pub views: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVideoRequest {
    pub title: String,
    pub description: Option<String>,
}

// Ensure this exactly matches what the frontend expects
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
}

#[derive(Clone)]
pub struct AppState {
    pub supabase_url: String,
    pub supabase_api_key: String,
    pub supabase_bucket: String,
}
