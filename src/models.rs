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

#[derive(Debug, Serialize, Deserialize)]
pub struct Show {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub release_date: String, // in ISO format: YYYY-MM-DD
    pub thumbnail_url: String,
    pub episode_count: i32,
    pub genre: String,
    pub rating: f32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShowResponse {
    pub id: String,
    pub title: String,
}
