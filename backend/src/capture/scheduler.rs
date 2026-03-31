use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::config::AppConfig;
use crate::models::Schedule;

use super::browser_recorder;
use super::recorder;

/// Background loop that checks for due schedules and starts recordings.
pub async fn run_scheduler(
    db: SqlitePool,
    config: AppConfig,
    token: CancellationToken,
) {
    let interval_secs = config.capture.retry_interval_secs.unwrap_or(15);
    let interval = std::time::Duration::from_secs(interval_secs);

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("Capture scheduler shutting down");
                break;
            }
            _ = tokio::time::sleep(interval) => {
                if let Err(err) = check_schedules(&db, &config, &token).await {
                    tracing::error!("Scheduler tick failed: {err:#}");
                }
            }
        }
    }
}

async fn check_schedules(
    db: &SqlitePool,
    config: &AppConfig,
    token: &CancellationToken,
) -> Result<()> {
    let now = Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    // Find pending schedules that are due (start_time <= now)
    let due_schedules = sqlx::query_as::<_, Schedule>(
        "SELECT * FROM schedules WHERE enabled = 1 AND status = 'pending' AND start_time <= ?1",
    )
    .bind(&now)
    .fetch_all(db)
    .await?;

    for schedule in &due_schedules {
        // Check if already missed (end_time has passed)
        if let Some(ref end_time) = schedule.end_time {
            if end_time.as_str() < now.as_str() {
                tracing::warn!(schedule_id = %schedule.id, "Schedule missed (end_time has passed)");
                sqlx::query(
                    "UPDATE schedules SET status = 'missed', updated_at = datetime('now') WHERE id = ?1",
                )
                .bind(&schedule.id)
                .execute(db)
                .await?;
                continue;
            }
        }

        if schedule.room_url.is_empty() && schedule.stream_url.is_empty() {
            tracing::warn!(schedule_id = %schedule.id, "Skipping schedule with no stream_url or room_url");
            continue;
        }

        // Set status to recording
        sqlx::query(
            "UPDATE schedules SET status = 'recording', updated_at = datetime('now') WHERE id = ?1",
        )
        .bind(&schedule.id)
        .execute(db)
        .await?;

        if !schedule.room_url.is_empty() {
            // Browser-based recording path
            tracing::info!(schedule_id = %schedule.id, title = %schedule.title, "Starting scheduled browser capture");
            browser_recorder::start_browser_recording(
                db.clone(),
                config.clone(),
                schedule.clone(),
                token.clone(),
            );
        } else {
            // Existing RTMP recording path
            tracing::info!(schedule_id = %schedule.id, title = %schedule.title, "Starting scheduled capture");
            recorder::start_recording(
                db.clone(),
                config.clone(),
                schedule.clone(),
                token.clone(),
            );
        }
    }

    // Handle completed recurring schedules — compute next occurrence
    let completed_recurring = sqlx::query_as::<_, Schedule>(
        "SELECT * FROM schedules WHERE enabled = 1 AND status = 'completed' AND recurrence IS NOT NULL",
    )
    .fetch_all(db)
    .await?;

    for schedule in &completed_recurring {
        if let Some(ref cron_expr) = schedule.recurrence {
            match advance_recurring_schedule(db, &schedule.id, cron_expr, &schedule.start_time, &schedule.end_time).await {
                Ok(()) => {
                    tracing::info!(schedule_id = %schedule.id, "Advanced recurring schedule to next occurrence");
                }
                Err(err) => {
                    tracing::error!(schedule_id = %schedule.id, "Failed to advance recurring schedule: {err:#}");
                }
            }
        }
    }

    Ok(())
}

async fn advance_recurring_schedule(
    db: &SqlitePool,
    schedule_id: &str,
    cron_expr: &str,
    current_start: &str,
    current_end: &Option<String>,
) -> Result<()> {
    let schedule: cron::Schedule = cron_expr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid cron expression: {e}"))?;

    let current_start_dt =
        chrono::NaiveDateTime::parse_from_str(current_start, "%Y-%m-%d %H:%M:%S")?;
    let current_start_utc = current_start_dt.and_utc();

    // Find next occurrence after the current start_time
    let next = schedule
        .after(&current_start_utc)
        .next()
        .ok_or_else(|| anyhow::anyhow!("No next occurrence found for cron expression"))?;

    let new_start = next.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    // If there was an end_time, shift it by the same delta
    let new_end = if let Some(ref end_str) = current_end {
        let end_dt = chrono::NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S")?;
        let duration = end_dt - current_start_dt;
        let new_end_dt = next.naive_utc() + duration;
        Some(new_end_dt.format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    sqlx::query(
        "UPDATE schedules SET start_time = ?1, end_time = ?2, status = 'pending', updated_at = datetime('now') WHERE id = ?3",
    )
    .bind(&new_start)
    .bind(&new_end)
    .bind(schedule_id)
    .execute(db)
    .await?;

    Ok(())
}
