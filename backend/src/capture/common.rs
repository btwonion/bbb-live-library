use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::bbb::importer::{generate_thumbnail, get_duration};
use crate::config::AppConfig;
use crate::models::Schedule;

/// Finalizes a recording by collecting metadata, inserting a DB row, and marking the schedule completed.
pub async fn finalize_recording(
    db: &SqlitePool,
    config: &AppConfig,
    schedule: &Schedule,
    id: &str,
    filename: &str,
    output_path: &str,
) -> Result<()> {
    let storage_dir = &config.capture.storage_dir;
    let format = config
        .capture
        .output_format
        .as_deref()
        .unwrap_or("mp4");

    // Get file size
    let file_size = tokio::fs::metadata(output_path)
        .await
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    // Get duration via ffprobe
    let duration = get_duration(&config.capture.ffmpeg_path, output_path)
        .await
        .unwrap_or(None);

    // Generate thumbnail
    let thumb_filename = format!("thumbs/{id}.jpg");
    let thumb_path = format!("{storage_dir}/{thumb_filename}");
    let thumbnail_path =
        match generate_thumbnail(&config.capture.ffmpeg_path, output_path, &thumb_path).await {
            Ok(()) => Some(thumb_filename),
            Err(err) => {
                tracing::warn!("Failed to generate thumbnail for {id}: {err}");
                None
            }
        };

    // Insert recording row
    sqlx::query(
        "INSERT INTO recordings (id, title, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, schedule_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'live_capture', ?8, datetime('now'), datetime('now'))",
    )
    .bind(id)
    .bind(&schedule.title)
    .bind(filename)
    .bind(&thumbnail_path)
    .bind(duration)
    .bind(file_size)
    .bind(format)
    .bind(&schedule.id)
    .execute(db)
    .await
    .context("Failed to insert recording into database")?;

    // Mark schedule completed
    set_schedule_status(db, &schedule.id, "completed").await?;

    tracing::info!(
        schedule_id = %schedule.id,
        recording_id = %id,
        "Recording completed successfully"
    );

    Ok(())
}

/// Updates the status of a schedule in the database.
pub async fn set_schedule_status(db: &SqlitePool, schedule_id: &str, status: &str) -> Result<()> {
    sqlx::query("UPDATE schedules SET status = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(status)
        .bind(schedule_id)
        .execute(db)
        .await
        .context("Failed to update schedule status")?;
    Ok(())
}
