use crate::api::error::AppError;
use crate::infrastructure::storage::local::{legacy::LegacyReader, LocalStorage};
use futures::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

struct TempFileGuard(PathBuf);

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Extract a single frame (at 1.0s, or 0.0s fallback) from a video file on disk as JPEG bytes.
pub async fn extract_video_thumbnail_from_file(video_path: &Path) -> Result<Vec<u8>, AppError> {
    // Try 1 second timestamp first (avoids black intro frame)
    if let Ok(bytes) = run_ffmpeg_thumbnail(video_path, "00:00:01").await {
        if !bytes.is_empty() {
            return Ok(bytes);
        }
    }
    // Fallback to 00:00:00 for short clips
    run_ffmpeg_thumbnail(video_path, "00:00:00").await
}

async fn run_ffmpeg_thumbnail(video_path: &Path, timestamp: &str) -> Result<Vec<u8>, AppError> {
    let temp_thumb_path =
        std::env::temp_dir().join(format!("fragrans_vthumb_{}.jpg", uuid::Uuid::new_v4()));
    let _guard = TempFileGuard(temp_thumb_path.clone());

    let cmd = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-v")
        .arg("error")
        .arg("-ss")
        .arg(timestamp)
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1")
        .arg("-vf")
        .arg("scale='min(360,iw)':-2")
        .arg("-q:v")
        .arg("2")
        .arg(&temp_thumb_path)
        .status();

    let status = tokio::time::timeout(std::time::Duration::from_secs(30), cmd)
        .await
        .map_err(|_| AppError::InternalError("ffmpeg thumbnail timed out".into()))?
        .map_err(|e| AppError::InternalError(format!("Failed to execute ffmpeg: {}", e)))?;

    if !status.success() || !temp_thumb_path.exists() {
        return Err(AppError::BadRequest("ffmpeg failed to extract thumbnail".into()));
    }

    let bytes = tokio::fs::read(&temp_thumb_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read thumbnail: {}", e)))?;

    if bytes.is_empty() {
        return Err(AppError::BadRequest("ffmpeg output is empty".into()));
    }

    Ok(bytes)
}

/// Extract thumbnail from a video stored in LocalStorage by decrypting its content.
pub async fn extract_video_thumbnail_from_storage(
    local_storage: &LocalStorage,
    user_id: &str,
    content_hash: &str,
) -> Result<Vec<u8>, AppError> {
    // If a transcoded 720p preview already exists, extracting from it is instantaneous
    if let Ok(preview_path) = local_storage.get_video_preview_path(user_id, content_hash) {
        if preview_path.exists() {
            if let Ok(bytes) = extract_video_thumbnail_from_file(&preview_path).await {
                return Ok(bytes);
            }
        }
    }

    let (_, _, mut stream) = local_storage
        .stream_content(user_id.to_string(), content_hash.to_string(), 0, None)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read source video: {}", e)))?;

    let temp_input_path =
        std::env::temp_dir().join(format!("fragrans_vin_{}.tmp", uuid::Uuid::new_v4()));
    let _input_guard = TempFileGuard(temp_input_path.clone());

    let mut temp_input_file = tokio::fs::File::create(&temp_input_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create temp input file: {}", e)))?;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res
            .map_err(|e| AppError::InternalError(format!("Failed to decrypt chunk: {}", e)))?;
        temp_input_file
            .write_all(&chunk)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write chunk: {}", e)))?;
    }

    temp_input_file
        .flush()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to flush temp file: {}", e)))?;
    drop(temp_input_file);

    extract_video_thumbnail_from_file(&temp_input_path).await
}

/// Extract thumbnail from a legacy video stored in LocalStorage.
pub async fn extract_video_thumbnail_from_legacy(
    local_storage: &LocalStorage,
    md5_hash: &str,
    iv: Option<&str>,
) -> Result<Vec<u8>, AppError> {
    let legacy = LegacyReader::new(local_storage);
    if let Ok(preview_path) = legacy.get_legacy_video_preview_path(md5_hash) {
        if preview_path.exists() {
            if let Ok(bytes) = extract_video_thumbnail_from_file(&preview_path).await {
                return Ok(bytes);
            }
        }
    }

    let (_, _, mut stream) = legacy
        .stream(md5_hash, iv, 0, None)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read legacy video: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Legacy video not found".to_string()))?;

    let temp_input_path =
        std::env::temp_dir().join(format!("fragrans_vin_{}.tmp", uuid::Uuid::new_v4()));
    let _input_guard = TempFileGuard(temp_input_path.clone());

    let mut temp_input_file = tokio::fs::File::create(&temp_input_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create temp input file: {}", e)))?;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res
            .map_err(|e| AppError::InternalError(format!("Failed to decrypt chunk: {}", e)))?;
        temp_input_file
            .write_all(&chunk)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write chunk: {}", e)))?;
    }

    temp_input_file
        .flush()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to flush temp file: {}", e)))?;
    drop(temp_input_file);

    extract_video_thumbnail_from_file(&temp_input_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_video_thumbnail_with_generated_clip() {
        let ffmpeg_check = tokio::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .await;
        if ffmpeg_check.is_err() {
            return;
        }

        let temp_vid =
            std::env::temp_dir().join(format!("test_src_{}.mp4", uuid::Uuid::new_v4()));
        let _vid_guard = TempFileGuard(temp_vid.clone());

        let status = tokio::process::Command::new("ffmpeg")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("testsrc=duration=2:size=320x240:rate=10")
            .arg("-y")
            .arg(&temp_vid)
            .status()
            .await
            .expect("ffmpeg lavfi command");

        assert!(status.success());
        assert!(temp_vid.exists());

        let thumb_bytes = extract_video_thumbnail_from_file(&temp_vid)
            .await
            .expect("extract thumbnail");

        assert!(!thumb_bytes.is_empty());
        assert_eq!(&thumb_bytes[0..2], &[0xFF, 0xD8]);
    }
}
