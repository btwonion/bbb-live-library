use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::models::Recording;

use super::client::BbbClient;

/// Result of a bulk BBB import run.
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub imported: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

/// Runs a bulk import of recordings from the BBB `getRecordings` API.
pub async fn run_bbb_import(db: &SqlitePool, config: &AppConfig) -> Result<ImportResult> {
    let client = BbbClient::new(&config.bbb);
    let bbb_recordings = client.get_recordings(None).await?;

    let mut result = ImportResult {
        imported: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    for bbb_rec in &bbb_recordings {
        // Check if already imported via bbb_meeting_id
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM recordings WHERE bbb_meeting_id = ?1",
        )
        .bind(&bbb_rec.record_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);

        if exists > 0 {
            result.skipped += 1;
            continue;
        }

        match import_single_bbb_recording(db, config, &client, bbb_rec).await {
            Ok(_) => result.imported += 1,
            Err(err) => {
                let msg = format!("Failed to import {}: {err:#}", bbb_rec.record_id);
                tracing::error!("{msg}");
                result.errors.push(msg);
            }
        }
    }

    Ok(result)
}

async fn import_single_bbb_recording(
    db: &SqlitePool,
    config: &AppConfig,
    client: &BbbClient,
    bbb_rec: &super::client::BbbRecording,
) -> Result<Recording> {
    let id = uuid::Uuid::new_v4().to_string();
    let ext = url_extension(&bbb_rec.playback_url).unwrap_or("mp4");
    let filename = format!("{id}.{ext}");
    let storage_dir = &config.capture.storage_dir;
    let dest = format!("{storage_dir}/{filename}");

    // Download video
    let file_size = client
        .download_file(&bbb_rec.playback_url, Path::new(&dest))
        .await
        .context("Failed to download recording video")?;

    // Get duration via ffprobe
    let duration = get_duration(&config.capture.ffmpeg_path, &dest)
        .await
        .unwrap_or(bbb_rec.duration);

    // Generate thumbnail
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

    let title = if bbb_rec.name.is_empty() {
        format!("BBB Recording {}", &bbb_rec.record_id)
    } else {
        bbb_rec.name.clone()
    };

    let recording = sqlx::query_as::<_, Recording>(
        "INSERT INTO recordings (id, title, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, bbb_meeting_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'bbb_import', ?8, datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(&title)
    .bind(&filename)
    .bind(&thumbnail_path)
    .bind(duration)
    .bind(file_size as i64)
    .bind(ext)
    .bind(&bbb_rec.record_id)
    .fetch_one(db)
    .await
    .context("Failed to insert recording into database")?;

    Ok(recording)
}

/// Imports a single recording from a user-provided URL.
pub async fn import_from_url(
    db: &SqlitePool,
    config: &AppConfig,
    url: &str,
    title: Option<&str>,
) -> Result<Recording> {
    let client = BbbClient::new(&config.bbb);
    let id = uuid::Uuid::new_v4().to_string();
    let ext = url_extension(url).unwrap_or("mp4");
    let filename = format!("{id}.{ext}");
    let storage_dir = &config.capture.storage_dir;
    let dest = format!("{storage_dir}/{filename}");

    let file_size = client
        .download_file(url, Path::new(&dest))
        .await
        .context("Failed to download video from URL")?;

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
        "INSERT INTO recordings (id, title, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'bbb_import', datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(title)
    .bind(&filename)
    .bind(&thumbnail_path)
    .bind(duration)
    .bind(file_size as i64)
    .bind(ext)
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
