use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::bbb::importer::{generate_thumbnail, get_duration};
use crate::config::AppConfig;
use crate::models::Schedule;

/// Spawns an ffmpeg process to record from `schedule.stream_url` and returns a task handle.
pub fn start_recording(
    db: SqlitePool,
    config: AppConfig,
    schedule: Schedule,
    token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(err) = run_recording(&db, &config, &schedule, &token).await {
            tracing::error!(
                schedule_id = %schedule.id,
                "Recording failed: {err:#}"
            );
            if let Err(update_err) = set_schedule_status(&db, &schedule.id, "missed").await {
                tracing::error!("Failed to update schedule status to missed: {update_err:#}");
            }
        }
    })
}

async fn run_recording(
    db: &SqlitePool,
    config: &AppConfig,
    schedule: &Schedule,
    token: &CancellationToken,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let format = config
        .capture
        .output_format
        .as_deref()
        .unwrap_or("mp4");
    let filename = format!("{id}.{format}");
    let storage_dir = &config.capture.storage_dir;
    let output_path = format!("{storage_dir}/{filename}");

    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        schedule.stream_url.clone(),
        "-c".to_string(),
        "copy".to_string(),
    ];

    // If end_time is set, limit duration with -t
    if let Some(ref end_time) = schedule.end_time {
        if let (Ok(start), Ok(end)) = (
            chrono::NaiveDateTime::parse_from_str(&schedule.start_time, "%Y-%m-%d %H:%M:%S"),
            chrono::NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M:%S"),
        ) {
            let duration_secs = (end - start).num_seconds();
            if duration_secs > 0 {
                args.push("-t".to_string());
                args.push(duration_secs.to_string());
            }
        }
    }

    args.push(output_path.clone());

    tracing::info!(
        schedule_id = %schedule.id,
        stream_url = %schedule.stream_url,
        output = %output_path,
        "Starting ffmpeg recording"
    );

    let mut child = tokio::process::Command::new(&config.capture.ffmpeg_path)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg")?;

    let cancelled = token.cancelled();

    let status = tokio::select! {
        result = child.wait() => {
            result.context("Failed to wait for ffmpeg")?
        }
        _ = cancelled => {
            tracing::warn!(schedule_id = %schedule.id, "Recording cancelled, killing ffmpeg");
            let _ = child.kill().await;
            // Try to finalize partial recording
            finalize_recording(db, config, schedule, &id, &filename, &output_path).await?;
            return Ok(());
        }
    };

    if !status.success() {
        anyhow::bail!("ffmpeg exited with status {status}");
    }

    finalize_recording(db, config, schedule, &id, &filename, &output_path).await
}

async fn finalize_recording(
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

async fn set_schedule_status(db: &SqlitePool, schedule_id: &str, status: &str) -> Result<()> {
    sqlx::query("UPDATE schedules SET status = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(status)
        .bind(schedule_id)
        .execute(db)
        .await
        .context("Failed to update schedule status")?;
    Ok(())
}
