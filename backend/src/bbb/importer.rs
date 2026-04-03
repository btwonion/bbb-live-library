use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::models::Recording;

use super::client::download_file;
use super::public::resolve_public_recording;

/// Imports a single recording from a user-provided direct video URL.
pub async fn import_from_url(
    db: &SqlitePool,
    config: &AppConfig,
    url: &str,
    title: Option<&str>,
) -> Result<Recording> {
    let id = uuid::Uuid::new_v4().to_string();
    let ext = url_extension(url).unwrap_or("mp4");
    let filename = format!("{id}.{ext}");
    let storage_dir = &config.capture.storage_dir;
    let dest = format!("{storage_dir}/{filename}");

    let mut file_size = download_file(url, Path::new(&dest))
        .await
        .context("Failed to download video from URL")?;

    let hash = compute_file_hash(&dest).await?;
    if is_duplicate_hash(db, &hash).await? {
        let _ = tokio::fs::remove_file(&dest).await;
        anyhow::bail!("A recording with the same file content has already been imported");
    }

    if ext == "mp4" {
        if let Err(err) = faststart_mp4(&config.capture.ffmpeg_path, &dest).await {
            tracing::warn!("Faststart processing failed for {id}, keeping original: {err}");
        } else {
            file_size = tokio::fs::metadata(&dest).await?.len();
        }
    }

    let duration = get_duration(&config.capture.ffmpeg_path, &dest).await.ok();

    let thumb_filename = format!("thumbs/{id}.jpg");
    let thumb_path = format!("{storage_dir}/{thumb_filename}");
    let thumbnail_path = match generate_thumbnail(&config.capture.ffmpeg_path, &dest, &thumb_path)
        .await
    {
        Ok(()) => Some(thumb_filename),
        Err(err) => {
            tracing::warn!("Failed to generate thumbnail for {id}: {err}");
            None
        }
    };

    let title = title.unwrap_or("Imported Recording");

    let recording = sqlx::query_as::<_, Recording>(
        "INSERT INTO recordings (id, title, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, file_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'url_import', ?8, datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(title)
    .bind(&filename)
    .bind(&thumbnail_path)
    .bind(duration)
    .bind(file_size as i64)
    .bind(ext)
    .bind(&hash)
    .fetch_one(db)
    .await
    .context("Failed to insert recording into database")?;

    Ok(recording)
}

/// Imports a recording from a public BBB server by resolving its metadata and video URL.
pub async fn import_public_bbb(
    db: &SqlitePool,
    config: &AppConfig,
    server_url: &str,
    record_id: &str,
    title_override: Option<&str>,
) -> Result<Recording> {
    let resolved = resolve_public_recording(server_url, record_id).await?;

    let id = uuid::Uuid::new_v4().to_string();
    let ext = url_extension(&resolved.video_url).unwrap_or("mp4");
    let filename = format!("{id}.{ext}");
    let storage_dir = &config.capture.storage_dir;
    let dest = format!("{storage_dir}/{filename}");

    let mut file_size = download_file(&resolved.video_url, Path::new(&dest))
        .await
        .context("Failed to download BBB recording video")?;

    let hash = compute_file_hash(&dest).await?;
    if is_duplicate_hash(db, &hash).await? {
        let _ = tokio::fs::remove_file(&dest).await;
        anyhow::bail!("A recording with the same file content has already been imported");
    }

    if ext == "mp4" {
        if let Err(err) = faststart_mp4(&config.capture.ffmpeg_path, &dest).await {
            tracing::warn!("Faststart processing failed for {id}, keeping original: {err}");
        } else {
            file_size = tokio::fs::metadata(&dest).await?.len();
        }
    }

    let duration = get_duration(&config.capture.ffmpeg_path, &dest).await.ok();

    let thumb_filename = format!("thumbs/{id}.jpg");
    let thumb_path = format!("{storage_dir}/{thumb_filename}");
    let thumbnail_path = match generate_thumbnail(&config.capture.ffmpeg_path, &dest, &thumb_path)
        .await
    {
        Ok(()) => Some(thumb_filename),
        Err(err) => {
            tracing::warn!("Failed to generate thumbnail for {id}: {err}");
            None
        }
    };

    let title = title_override.unwrap_or(&resolved.meeting_name);

    let recording = sqlx::query_as::<_, Recording>(
        "INSERT INTO recordings (id, title, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, file_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'bbb_public', ?8, datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(title)
    .bind(&filename)
    .bind(&thumbnail_path)
    .bind(duration)
    .bind(file_size as i64)
    .bind(ext)
    .bind(&hash)
    .fetch_one(db)
    .await
    .context("Failed to insert recording into database")?;

    Ok(recording)
}

/// Gets the duration of a media file in seconds using ffprobe.
pub(crate) async fn get_duration(ffmpeg_path: &str, file_path: &str) -> Result<Option<i64>> {
    let ffprobe_path = ffmpeg_path.replace("ffmpeg", "ffprobe");

    let output = tokio::process::Command::new(&ffprobe_path)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file_path,
        ])
        .output()
        .await
        .context("Failed to run ffprobe")?;

    if !output.status.success() {
        anyhow::bail!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration = stdout
        .trim()
        .parse::<f64>()
        .ok()
        .map(|d| d.round() as i64);

    Ok(duration)
}

/// Generates a thumbnail image from a video file using ffmpeg.
pub(crate) async fn generate_thumbnail(ffmpeg_path: &str, input: &str, output: &str) -> Result<()> {
    let status = tokio::process::Command::new(ffmpeg_path)
        .args([
            "-y",
            "-i",
            input,
            "-ss",
            "00:00:05",
            "-frames:v",
            "1",
            "-q:v",
            "2",
            output,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .context("Failed to run ffmpeg for thumbnail")?;

    if !status.success() {
        anyhow::bail!("ffmpeg thumbnail generation failed");
    }

    Ok(())
}

/// Rewrites an MP4 file with the moov atom at the start for efficient browser streaming.
async fn faststart_mp4(ffmpeg_path: &str, file_path: &str) -> Result<()> {
    let tmp = format!("{file_path}.faststart.tmp");
    let status = tokio::process::Command::new(ffmpeg_path)
        .args([
            "-y",
            "-i",
            file_path,
            "-c",
            "copy",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
            &tmp,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .context("Failed to run ffmpeg for faststart")?;

    if !status.success() {
        let _ = tokio::fs::remove_file(&tmp).await;
        anyhow::bail!("ffmpeg faststart processing failed");
    }

    tokio::fs::rename(&tmp, file_path)
        .await
        .context("Failed to replace file with faststart version")?;

    Ok(())
}

/// Computes the SHA-256 hash of a file, returning a hex string.
async fn compute_file_hash(path: &str) -> Result<String> {
    let data = tokio::fs::read(path)
        .await
        .context("Failed to read file for hashing")?;
    let hash = Sha256::digest(&data);
    Ok(hash.iter().map(|b| format!("{b:02x}")).collect())
}

/// Checks whether a recording with the given file hash already exists.
async fn is_duplicate_hash(db: &SqlitePool, hash: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM recordings WHERE file_hash = ?1",
    )
    .bind(hash)
    .fetch_one(db)
    .await
    .unwrap_or(0);
    Ok(count > 0)
}

fn url_extension(url: &str) -> Option<&str> {
    let path = url.split('?').next()?;
    let filename = path.rsplit('/').next()?;
    let ext = filename.rsplit('.').next()?;
    if ext.len() <= 5 && ext != filename {
        Some(ext)
    } else {
        None
    }
}
