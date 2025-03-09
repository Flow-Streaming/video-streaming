use axum::http::StatusCode;
use postgrest::Postgrest;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use crate::models::AppState;

pub struct SupabaseService {
    state: Arc<AppState>,
    client: Client,
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

    pub async fn query_single<T: DeserializeOwned>(
        &self,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<T, (StatusCode, String)> {
        let response = self
            .postgrest_client()
            .from(table)
            .select("*")
            .eq(column, value)
            .single()
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if response.status() == 406 {
            return Err((StatusCode::NOT_FOUND, format!("{} not found", table)));
        }

        let data: T = response
            .text()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            .and_then(|text| {
                serde_json::from_str(&text)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            })?;

        Ok(data)
    }

    pub async fn query_all<T: DeserializeOwned>(
        &self,
        table: &str,
        order_by: Option<&str>,
    ) -> Result<Vec<T>, (StatusCode, String)> {
        let mut query = self.postgrest_client().from(table).select("*");

        if let Some(order) = order_by {
            query = query.order(order);
        }

        let response = query
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let data: Vec<T> = response
            .text()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            .and_then(|text| {
                serde_json::from_str(&text)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            })?;

        Ok(data)
    }

    pub async fn insert<T>(
        &self,
        table: &str,
        data: serde_json::Value,
    ) -> Result<(), (StatusCode, String)> {
        self.postgrest_client()
            .from(table)
            .insert(data.to_string())
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    pub async fn delete(
        &self,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<(), (StatusCode, String)> {
        self.postgrest_client()
            .from(table)
            .delete()
            .eq(column, value)
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    pub async fn upload_file(
        &self,
        bucket: &str,
        path: &str,
        content: Vec<u8>,
        content_type: &str,
    ) -> Result<(), (StatusCode, String)> {
        let storage_url = format!(
            "{}/storage/v1/object/{}/{}",
            self.state.supabase_url, bucket, path
        );

        let response = self
            .client
            .post(&storage_url)
            .header("apikey", &self.state.supabase_api_key)
            .header("Content-Type", content_type)
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

    pub async fn delete_file(&self, bucket: &str, path: &str) -> Result<(), (StatusCode, String)> {
        let storage_url = format!(
            "{}/storage/v1/object/{}/{}",
            self.state.supabase_url, bucket, path
        );

        self.client
            .delete(&storage_url)
            .header("apikey", &self.state.supabase_api_key)
            .send()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    pub fn get_public_url(&self, bucket: &str, path: &str) -> String {
        format!(
            "{}/storage/v1/object/public/{}/{}",
            self.state.supabase_url, bucket, path
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

    pub async fn call_rpc(
        &self,
        function: &str,
        params: serde_json::Value,
    ) -> Result<(), (StatusCode, String)> {
        self.postgrest_client()
            .rpc(function, &params.to_string())
            .execute()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }
}
