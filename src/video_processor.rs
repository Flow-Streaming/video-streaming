use anyhow::Result;
use axum::http::StatusCode;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

pub struct VideoProcessor;

impl VideoProcessor {
    /// Process a video using FFmpeg and return the path to the processed file
    pub async fn process_video(
        video_data: &[u8],
        filename: &str,
    ) -> Result<(String, String), (StatusCode, String)> {
        // Generate a unique ID for this video
        let video_id = Uuid::new_v4().to_string();

        // Create temporary file for the input
        let mut input_file = NamedTempFile::new().map_err(|e| {
            error!("Failed to create temp file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create temporary file".to_string(),
            )
        })?;

        // Write video data to the temp file
        input_file.write_all(video_data).map_err(|e| {
            error!("Failed to write to temp file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to write to temporary file".to_string(),
            )
        })?;

        // Get the path of the input file
        let input_path = input_file.path().to_str().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid temporary file path".to_string(),
            )
        })?;

        // Create temp file for the output
        let output_file = NamedTempFile::new().map_err(|e| {
            error!("Failed to create output temp file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create output temporary file".to_string(),
            )
        })?;

        let output_path = output_file.path().to_str().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid output file path".to_string(),
            )
        })?;

        // Create temp file for the thumbnail
        let thumbnail_file = NamedTempFile::new().map_err(|e| {
            error!("Failed to create thumbnail temp file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create thumbnail temporary file".to_string(),
            )
        })?;

        let thumbnail_path = thumbnail_file.path().to_str().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid thumbnail file path".to_string(),
            )
        })?;

        // Process the video (compress and convert to MP4)
        let process_result = Command::new("ffmpeg")
            .arg("-i")
            .arg(input_path)
            .arg("-c:v")
            .arg("libx264")
            .arg("-crf")
            .arg("23") // Compression quality (lower = better quality, higher = smaller file)
            .arg("-preset")
            .arg("medium") // Encoding speed/compression trade-off
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("128k")
            .arg("-y") // Overwrite output file if it exists
            .arg(output_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| {
                error!("FFmpeg process error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to process video: {}", e),
                )
            })?;

        if !process_result.success() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "FFmpeg processing failed".to_string(),
            ));
        }

        // Generate thumbnail from the first frame
        let thumbnail_result = Command::new("ffmpeg")
            .arg("-i")
            .arg(input_path)
            .arg("-ss")
            .arg("00:00:01") // 1 second into the video
            .arg("-vframes")
            .arg("1")
            .arg("-y")
            .arg(thumbnail_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| {
                error!("Thumbnail generation error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to generate thumbnail: {}", e),
                )
            })?;

        if !thumbnail_result.success() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Thumbnail generation failed".to_string(),
            ));
        }

        // Get the base name without extension
        let base_name = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");

        // Create filenames for upload
        let processed_filename = format!("{}-{}.mp4", base_name, video_id);
        let thumbnail_filename = format!("{}-{}-thumbnail.jpg", base_name, video_id);

        info!(
            "Video processed successfully: {} and thumbnail: {}",
            processed_filename, thumbnail_filename
        );

        // Read the processed video and thumbnail
        let processed_video = fs::read(output_path).await.map_err(|e| {
            error!("Failed to read processed video: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read processed video".to_string(),
            )
        })?;

        let thumbnail_data = fs::read(thumbnail_path).await.map_err(|e| {
            error!("Failed to read thumbnail: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read thumbnail".to_string(),
            )
        })?;

        Ok((processed_filename, thumbnail_filename))
    }
}
