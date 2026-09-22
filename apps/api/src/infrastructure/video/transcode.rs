use crate::infrastructure::storage::local::{LocalStorage, legacy::LegacyReader};
use futures::StreamExt;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

static TRANSCODE_SEMAPHORE: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(1)));
static IN_PROGRESS_TRANSCODES: LazyLock<Arc<Mutex<HashSet<String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashSet::new())));

struct InProgressGuard {
    key: String,
}

impl Drop for InProgressGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = IN_PROGRESS_TRANSCODES.lock() {
            set.remove(&self.key);
        }
    }
}

struct TempFileGuard(PathBuf);

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub async fn ensure_video_preview(
    local_storage: &LocalStorage,
    user_id: &str,
    content_hash: Option<&str>,
    md5_hash: Option<&str>,
    iv: Option<&str>,
) -> Result<(), String> {
    let (key, preview_path) = if let Some(hash) = content_hash {
        let path = local_storage
            .get_video_preview_path(user_id, hash)
            .map_err(|e| e.to_string())?;
        (format!("sha256:{}", hash), path)
    } else if let Some(md5) = md5_hash {
        let path = LegacyReader::new(local_storage)
            .get_legacy_video_preview_path(md5)
            .map_err(|e| e.to_string())?;
        (format!("md5:{}", md5), path)
    } else {
        return Err("Neither content_hash nor md5_hash provided".to_string());
    };

    if preview_path.exists() {
        return Ok(());
    }

    // Skip transcoding for files larger than 2GB to protect disk and CPU
    if let Some(hash) = content_hash {
        if let Ok(size) = local_storage.get_plaintext_size(user_id, hash).await {
            if size > 2 * 1024 * 1024 * 1024 {
                tracing::info!(
                    user_id = %user_id,
                    content_hash = %hash,
                    size = %size,
                    "Video exceeds 2GB, skipping background 720p transcode"
                );
                return Ok(());
            }
        }
    }

    {
        let mut in_progress = IN_PROGRESS_TRANSCODES
            .lock()
            .map_err(|e| e.to_string())?;
        if !in_progress.insert(key.clone()) {
            // Already being transcoded by another worker
            return Ok(());
        }
    }

    let _guard = InProgressGuard { key };

    let _permit = TRANSCODE_SEMAPHORE
        .acquire()
        .await
        .map_err(|e| e.to_string())?;

    if preview_path.exists() {
        return Ok(());
    }

    let temp_input_path =
        std::env::temp_dir().join(format!("fragrans_vin_{}.tmp", uuid::Uuid::new_v4()));
    let _input_guard = TempFileGuard(temp_input_path.clone());

    let mut temp_input_file = tokio::fs::File::create(&temp_input_path)
        .await
        .map_err(|e| format!("Failed to create temp input file: {}", e))?;

    if let Some(hash) = content_hash {
        let (_, _, mut stream) = local_storage
            .stream_content(user_id.to_string(), hash.to_string(), 0, None)
            .await
            .map_err(|e| format!("Failed to read source video: {}", e))?;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| format!("Failed to decrypt chunk: {}", e))?;
            temp_input_file
                .write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write decrypted chunk: {}", e))?;
        }
    } else if let Some(md5) = md5_hash {
        let legacy = LegacyReader::new(local_storage);
        let (_, _, mut stream) = legacy
            .stream(md5, iv, 0, None)
            .await
            .map_err(|e| format!("Failed to read legacy video: {}", e))?
            .ok_or_else(|| "Legacy video not found".to_string())?;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| format!("Failed to decrypt chunk: {}", e))?;
            temp_input_file
                .write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write decrypted chunk: {}", e))?;
        }
    }

    temp_input_file
        .flush()
        .await
        .map_err(|e| format!("Failed to flush temp input file: {}", e))?;
    drop(temp_input_file);

    let temp_output_path =
        preview_path.with_extension(format!("tmp_{}.mp4", uuid::Uuid::new_v4()));
    let _output_guard = TempFileGuard(temp_output_path.clone());

    let ffmpeg_fut = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-v")
        .arg("error")
        .arg("-i")
        .arg(&temp_input_path)
        .arg("-vf")
        .arg("scale=-2:'min(720,ih)'")
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-preset")
        .arg("veryfast")
        .arg("-crf")
        .arg("24")
        .arg("-maxrate")
        .arg("1500k")
        .arg("-bufsize")
        .arg("3000k")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg("-movflags")
        .arg("+faststart")
        .arg(&temp_output_path)
        .output();

    let output = tokio::time::timeout(std::time::Duration::from_secs(600), ffmpeg_fut)
        .await
        .map_err(|_| "ffmpeg transcode timed out after 600 seconds".to_string())?;

    match output {
        Ok(out) if out.status.success() => {
            if let Some(hash) = content_hash {
                local_storage
                    .store_video_preview_encrypted(user_id, hash, &temp_output_path)
                    .await
                    .map_err(|e| format!("Failed to encrypt preview file: {}", e))?;
            } else {
                if let Some(parent) = preview_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                tokio::fs::rename(&temp_output_path, &preview_path)
                    .await
                    .map_err(|e| format!("Failed to rename preview file: {}", e))?;
            }
            tracing::info!(
                path = %preview_path.display(),
                "Video preview generated and stored successfully"
            );
            Ok(())
        }
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(
                error = %err_msg,
                "ffmpeg failed to transcode video preview"
            );
            Err(format!("ffmpeg failed: {}", err_msg))
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to execute ffmpeg command");
            Err(format!("ffmpeg exec error: {}", e))
        }
    }
}
