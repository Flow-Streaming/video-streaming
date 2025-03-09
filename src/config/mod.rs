use std::sync::Arc;
use dotenv::dotenv;
use crate::models::AppState;

pub fn load_config() -> Arc<AppState> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize application state
    Arc::new(AppState {
        supabase_url: std::env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set"),
        supabase_api_key: std::env::var("SUPABASE_API_KEY")
            .expect("SUPABASE_API_KEY must be set"),
        supabase_bucket: std::env::var("SUPABASE_BUCKET")
            .unwrap_or_else(|_| "videos".to_string()),
    })
}