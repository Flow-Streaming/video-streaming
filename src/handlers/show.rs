use std::sync::Arc;

use anyhow::Result;
use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    models::{AppState, CreateShowResponse, Show},
    supabase,
};

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
    let supabase = supabase::SupabaseService::new(state).await?;

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
