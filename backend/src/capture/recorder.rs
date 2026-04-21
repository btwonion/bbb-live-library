use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::common::{finalize_recording, graceful_stop_ffmpeg, set_schedule_status};
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

    // If end_time is set, limit duration with -t (including offsets)
    if let Some(ref end_time) = schedule.end_time {
        if let (Ok(start), Ok(end)) = (
            chrono::NaiveDateTime::parse_from_str(&schedule.start_time, "%Y-%m-%d %H:%M:%S"),
            chrono::NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M:%S"),
        ) {
            let duration_secs = (end - start).num_seconds() + schedule.start_offset_secs + schedule.end_offset_secs;
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
            tracing::warn!(schedule_id = %schedule.id, "Recording cancelled, stopping ffmpeg gracefully");
            graceful_stop_ffmpeg(&mut child).await;
            // Try to finalize partial recording
            finalize_recording(db, config, schedule, &id, &filename, &output_path).await?;
            return Ok(());
        }
    };

    if !status.success() {
        anyhow::bail!("ffmpeg exited with status {status}");
    }

    // If finalization fails (e.g. ffprobe, thumbnail), still mark as completed since the file exists
    if let Err(err) = finalize_recording(db, config, schedule, &id, &filename, &output_path).await {
        tracing::error!(schedule_id = %schedule.id, "Finalization failed but recording was captured: {err:#}");
        set_schedule_status(db, &schedule.id, "completed").await?;
    }
    Ok(())
}
