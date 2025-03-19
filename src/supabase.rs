use axum::http::StatusCode;
use postgrest::Postgrest;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::{error, info};

use crate::{config::VIDEO_BUCKET, models::AppState};

pub struct SupabaseService {
    pub state: Arc<AppState>,
    pub client: Client,
}

impl SupabaseService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            client: Client::new(),
        }
    }

    pub fn postgrest_client(&self) -> Postgrest {
        Postgrest::new(&self.state.supabase_url)
            .insert_header("apikey", &self.state.supabase_api_key)
    }

    // pub async fn query_single<T: DeserializeOwned>(
    //     &self,
    //     table: &str,
    //     column: &str,
    //     value: &str,
    // ) -> Result<T, (StatusCode, String)> {
    //     // Let's use the direct HTTP client approach for more control and better debugging
    //     let url = format!(
    //         "{}/rest/v1/{}?{}=eq.{}&select=*",
    //         self.state.supabase_url, table, column, value
    //     );

    //     info!("Making request to: {}", url);

    //     let response = self
    //         .client
    //         .get(&url)
    //         .header("apikey", &self.state.supabase_api_key)
    //         .header(
    //             "Authorization",
    //             format!("Bearer {}", self.state.supabase_api_key),
    //         )
    //         .header("Content-Type", "application/json")
    //         .header("Accept", "application/json")
    //         .send()
    //         .await
    //         .map_err(|e| {
    //             error!("Request error: {}", e);
    //             (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    //         })?;

    //     let status = response.status();
    //     info!("Response status: {}", status);

    //     let body = response.text().await.map_err(|e| {
    //         error!("Error reading response body: {}", e);
    //         (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    //     })?;

    //     info!("Response body: {}", body);

    //     if !status.is_success() {
    //         return Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Supabase API error: {} - {}", status, body),
    //         ));
    //     }

    //     // Handle empty responses
    //     if body.trim().is_empty() || body == "[]" {
    //         return Err((
    //             StatusCode::NOT_FOUND,
    //             format!("{} with {} = {} not found", table, column, value),
    //         ));
    //     }

    //     // Parse the response - Supabase returns an array even for single items when using ?single=true
    //     let parsed: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
    //         tracing::error!("JSON parse error: {}", e);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("JSON parse error: {}", e),
    //         )
    //     })?;

    //     // If it's an array with one item, extract the item
    //     let item_value = if parsed.is_array() {
    //         let array = parsed.as_array().unwrap();
    //         if array.is_empty() {
    //             return Err((
    //                 StatusCode::NOT_FOUND,
    //                 format!("{} with {} = {} not found", table, column, value),
    //             ));
    //         }
    //         array[0].clone()
    //     } else {
    //         parsed
    //     };

    //     // Deserialize the single object
    //     serde_json::from_value::<T>(item_value.clone()).map_err(|e| {
    //         error!("Deserialization error: {:?}", e);
    //         error!("JSON value: {:?}", item_value.clone());
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!(
    //                 "Failed to deserialize response: {} - Value: {}",
    //                 e, item_value
    //             ),
    //         )
    //     })
    // }

    // pub async fn query_all<T>(
    //     &self,
    //     table: &str,
    //     order: Option<&str>,
    // ) -> Result<Vec<T>, (StatusCode, String)>
    // where
    //     T: serde::de::DeserializeOwned,
    // {
    //     // Build the URL with optional ordering
    //     let mut url = format!("{}/rest/v1/{}?select=*", self.state.supabase_url, table);

    //     if let Some(order_by) = order {
    //         url = format!("{}&order={}", url, order_by);
    //     }

    //     // Make the request to Supabase
    //     let response = self
    //         .client
    //         .get(&url)
    //         .header("apikey", &self.state.supabase_api_key)
    //         .header(
    //             "Authorization",
    //             &format!("Bearer {}", self.state.supabase_api_key),
    //         )
    //         .send()
    //         .await
    //         .map_err(|e| {
    //             (
    //                 StatusCode::INTERNAL_SERVER_ERROR,
    //                 format!("Request error: {}", e),
    //             )
    //         })?;

    //     // Check if the response is successful
    //     if !response.status().is_success() {
    //         let error_text = response.text().await.unwrap_or_default();
    //         println!("supabase error");
    //         return Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Supabase error: {}", error_text),
    //         ));
    //     }

    //     // Get the response body as text for flexible parsing
    //     let body_text = response.text().await.map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Reading response error: {}", e),
    //         )
    //     })?;

    //     // If response is empty, return empty vector
    //     if body_text.trim().is_empty() {
    //         return Ok(Vec::new());
    //     }

    //     // Try to parse as a JSON value first to determine structure
    //     let json_value: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("JSON parse error: {}", e),
    //         )
    //     })?;

    //     // Handle different response formats
    //     if json_value.is_array() {
    //         // Direct array format (most common)
    //         let items: Vec<T> = serde_json::from_value(json_value).map_err(|e| {
    //             (
    //                 StatusCode::INTERNAL_SERVER_ERROR,
    //                 format!("Array deserialization error: {}", e),
    //             )
    //         })?;
    //         Ok(items)
    //     } else if json_value.is_object() {
    //         // Handle object format
    //         if let Some(data) = json_value.get("data") {
    //             if data.is_array() {
    //                 // Object with data array field
    //                 let items: Vec<T> = serde_json::from_value(data.clone()).map_err(|e| {
    //                     (
    //                         StatusCode::INTERNAL_SERVER_ERROR,
    //                         format!("Object data deserialization error: {}", e),
    //                     )
    //                 })?;
    //                 return Ok(items);
    //             }
    //         }

    //         // If object is empty or doesn't have expected format but is valid JSON
    //         if json_value.as_object().map_or(false, |obj| obj.is_empty()) {
    //             return Ok(Vec::new());
    //         }

    //         // Try treating the object itself as a single item
    //         match serde_json::from_value::<T>(json_value.clone()) {
    //             Ok(item) => Ok(vec![item]),
    //             Err(_) => {
    //                 // Last resort: Try to extract array from another field or just return the error
    //                 Err((
    //                     StatusCode::INTERNAL_SERVER_ERROR,
    //                     format!(
    //                         "Unexpected response format. Expected array, got object: {}",
    //                         body_text
    //                     ),
    //                 ))
    //             }
    //         }
    //     } else {
    //         // Neither array nor object (shouldn't happen with valid JSON)
    //         Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Unexpected JSON value type: {}", body_text),
    //         ))
    //     }
    // }

    // pub async fn insert<T>(
    //     &self,
    //     table: &str,
    //     data: serde_json::Value,
    // ) -> Result<(), (StatusCode, String)> {
    //     // Direct API call approach for better debugging
    //     let url = format!("{}/rest/v1/{}", self.state.supabase_url, table);

    //     let response = self
    //         .client
    //         .post(&url)
    //         .header("apikey", &self.state.supabase_api_key)
    //         .header(
    //             "Authorization",
    //             format!("Bearer {}", self.state.supabase_api_key),
    //         )
    //         .header("Content-Type", "application/json")
    //         .header("Prefer", "return=minimal")
    //         .json(&data)
    //         .send()
    //         .await
    //         .map_err(|e| {
    //             (
    //                 StatusCode::INTERNAL_SERVER_ERROR,
    //                 format!("Request error: {}", e),
    //             )
    //         })?;

    //     if !response.status().is_success() {
    //         // let error_text = response.text().await.unwrap_or_default();
    //         return Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!(
    //                 "Supabase insert error: {} - {}",
    //                 response.status(),
    //                 response.text().await.unwrap_or_default()
    //             ),
    //         ));
    //     }

    //     Ok(())
    // }

    // pub async fn delete(
    //     &self,
    //     table: &str,
    //     column: &str,
    //     value: &str,
    // ) -> Result<(), (StatusCode, String)> {
    //     self.postgrest_client()
    //         .from(table)
    //         .delete()
    //         .eq(column, value)
    //         .execute()
    //         .await
    //         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    //     Ok(())
    // }

    pub async fn upload_file(
        &self,
        file_name: &str,
        content: Vec<u8>,
    ) -> Result<(), (StatusCode, String)> {
        let storage_url = format!(
            "{}/storage/v1/object/{}/{}",
            self.state.supabase_url, VIDEO_BUCKET, file_name
        );

        let response = self
            .client
            .post(&storage_url)
            .header("apikey", &self.state.supabase_api_key)
            .header("Content-Type", "video/mp4")
            .body(content)
            .send()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if !response.status().is_success() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to upload to storage: {}", response.status()),
            ));
        }

        Ok(())
    }

    pub async fn delete_file(&self, file_name: &str) -> Result<(), (StatusCode, String)> {
        let storage_url = format!(
            "{}/storage/v1/object/{}/{}",
            self.state.supabase_url, VIDEO_BUCKET, file_name
        );

        self.client
            .delete(&storage_url)
            .header("apikey", &self.state.supabase_api_key)
            .send()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    pub fn get_public_url(&self, file_name: &str) -> String {
        format!(
            "{}/storage/v1/object/public/{}/{}",
            self.state.supabase_url, VIDEO_BUCKET, file_name
        )
    }

    pub async fn update(
        &self,
        table: &str,
        column: &str,
        value: &str,
        data: serde_json::Value,
    ) -> Result<(), (StatusCode, String)> {
        self.postgrest_client()
            .from(table)
            .update(data.to_string())
            .eq(column, value)
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    // pub async fn call_rpc(
    //     &self,
    //     function: &str,
    //     params: serde_json::Value,
    // ) -> Result<(), (StatusCode, String)> {
    //     self.postgrest_client()
    //         .rpc(function, &params.to_string())
    //         .execute()
    //         .await
    //         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    //     Ok(())
    // }
}
