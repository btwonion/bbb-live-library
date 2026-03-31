use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::common::{finalize_recording, set_schedule_status};
use crate::config::AppConfig;
use crate::models::Schedule;

/// Spawns a browser-based recording pipeline and returns a task handle.
///
/// The pipeline consists of Xvfb (virtual display), PulseAudio (virtual audio),
/// a Playwright recorder script (browser automation), and ffmpeg (screen + audio capture).
pub fn start_browser_recording(
    db: SqlitePool,
    config: AppConfig,
    schedule: Schedule,
    token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(err) = run_browser_recording(&db, &config, &schedule, &token).await {
            tracing::error!(
                schedule_id = %schedule.id,
                "Browser recording failed: {err:#}"
            );
            if let Err(update_err) = set_schedule_status(&db, &schedule.id, "missed").await {
                tracing::error!("Failed to update schedule status to missed: {update_err:#}");
            }
        }
    })
}

/// Polls a BBB room URL until the meeting appears to be active.
async fn poll_until_meeting_active(room_url: &str, timeout: Duration) -> Result<()> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if tokio::time::Instant::now() >= deadline {
            bail!("Timed out waiting for meeting to become active at {room_url}");
        }

        match client.get(room_url).send().await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    let body_lower = body.to_lowercase();
                    // Meeting is likely active if we see a join button or no "not started" message
                    let has_join_indicator = body_lower.contains("join-btn")
                        || body_lower.contains("join_name")
                        || body_lower.contains("type=\"submit\"")
                        || body_lower.contains("join meeting");
                    let has_not_started = body_lower.contains("not started")
                        || body_lower.contains("not running")
                        || body_lower.contains("meeting hasn't started")
                        || body_lower.contains("wait for moderator");

                    if has_join_indicator && !has_not_started {
                        tracing::info!("Meeting appears active at {room_url}");
                        return Ok(());
                    }

                    tracing::debug!(
                        "Meeting not yet active at {room_url} (join={has_join_indicator}, not_started={has_not_started})"
                    );
                }
            }
            Err(err) => {
                tracing::debug!("Failed to poll room URL {room_url}: {err}");
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

/// Derives a unique X display number from the schedule ID.
fn display_number_for_schedule(schedule_id: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    schedule_id.hash(&mut hasher);
    // Use displays 99–198 to avoid collisions with real displays
    99 + (hasher.finish() % 100) as u32
}

/// Computes recording duration in seconds from schedule start/end times.
fn compute_duration_secs(schedule: &Schedule) -> Option<i64> {
    schedule.end_time.as_ref().and_then(|end_time| {
        let start =
            chrono::NaiveDateTime::parse_from_str(&schedule.start_time, "%Y-%m-%d %H:%M:%S")
                .ok()?;
        let end =
            chrono::NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M:%S").ok()?;
        let secs = (end - start).num_seconds();
        if secs > 0 {
            Some(secs)
        } else {
            None
        }
    })
}

/// Kills a child process, logging any errors.
async fn kill_process(name: &str, child: &mut tokio::process::Child) {
    if let Err(err) = child.kill().await {
        tracing::debug!("Failed to kill {name}: {err}");
    }
}

async fn run_browser_recording(
    db: &SqlitePool,
    config: &AppConfig,
    schedule: &Schedule,
    token: &CancellationToken,
) -> Result<()> {
    let display_num = display_number_for_schedule(&schedule.id);
    let x_display = format!(":{display_num}");
    let pulse_runtime = format!("/tmp/pulse-bbb-{display_num}");
    let duration_secs = compute_duration_secs(schedule);

    tracing::info!(
        schedule_id = %schedule.id,
        room_url = %schedule.room_url,
        x_display = %x_display,
        "Starting browser recording pipeline"
    );

    // Poll until the meeting is active (up to 10 minutes or until end_time)
    let poll_timeout = duration_secs
        .map(|d| Duration::from_secs(d as u64))
        .unwrap_or(Duration::from_secs(600));

    poll_until_meeting_active(&schedule.room_url, poll_timeout).await?;

    // --- Step 1: Start Xvfb ---
    let mut xvfb = tokio::process::Command::new("Xvfb")
        .args([
            &*x_display,
            "-screen",
            "0",
            "1920x1080x24",
            "-nolisten",
            "tcp",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn Xvfb")?;

    // Give Xvfb a moment to initialize
    tokio::time::sleep(Duration::from_secs(1)).await;

    tracing::debug!(schedule_id = %schedule.id, x_display = %x_display, "Xvfb started");

    // --- Step 2: Start PulseAudio with virtual sink ---
    // Create runtime dir for PulseAudio
    let _ = tokio::fs::create_dir_all(&pulse_runtime).await;

    let _ = tokio::process::Command::new("pulseaudio")
        .args(["--start", "--exit-idle-time=-1"])
        .env("DISPLAY", &x_display)
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    // Load virtual audio sink
    let _ = tokio::process::Command::new("pactl")
        .args(["load-module", "module-null-sink", "sink_name=virtual_sink"])
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    let _ = tokio::process::Command::new("pactl")
        .args(["set-default-sink", "virtual_sink"])
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    tracing::debug!(schedule_id = %schedule.id, "PulseAudio configured");

    // --- Step 3: Spawn recorder script ---
    let recorder_script = config
        .capture
        .recorder_script_path
        .as_deref()
        .unwrap_or("recorder/record.js");

    let mut recorder_args = vec![
        recorder_script.to_string(),
        "--room-url".to_string(),
        schedule.room_url.clone(),
        "--bot-name".to_string(),
        schedule.bot_name.clone(),
        "--display".to_string(),
        x_display.clone(),
    ];

    if let Some(secs) = duration_secs {
        recorder_args.push("--timeout".to_string());
        recorder_args.push(secs.to_string());
    }

    let mut recorder = tokio::process::Command::new("node")
        .args(&recorder_args)
        .env("DISPLAY", &x_display)
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn recorder script")?;

    // Read stdout for RECORDING_STARTED signal
    let recorder_stdout = recorder.stdout.take().context("No stdout from recorder")?;
    let mut reader = BufReader::new(recorder_stdout).lines();

    // Forward stderr in a background task
    let recorder_stderr = recorder.stderr.take();
    let schedule_id_clone = schedule.id.clone();
    if let Some(stderr) = recorder_stderr {
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::debug!(schedule_id = %schedule_id_clone, "[recorder] {line}");
            }
        });
    }

    // Wait for RECORDING_STARTED with timeout
    let started = tokio::time::timeout(Duration::from_secs(120), async {
        while let Some(line) = reader.next_line().await? {
            if line.trim() == "RECORDING_STARTED" {
                return Ok(true);
            }
            if line.trim() == "RECORDING_STOPPED" {
                return Ok(false);
            }
        }
        Ok::<bool, std::io::Error>(false)
    })
    .await
    .context("Timed out waiting for recorder to join meeting")?
    .context("Error reading recorder stdout")?;

    if !started {
        kill_process("recorder", &mut recorder).await;
        kill_process("xvfb", &mut xvfb).await;
        bail!("Recorder script exited without starting");
    }

    tracing::info!(schedule_id = %schedule.id, "Recorder joined meeting, starting ffmpeg capture");

    // --- Step 4: Start ffmpeg capture ---
    let id = uuid::Uuid::new_v4().to_string();
    let format = config.capture.output_format.as_deref().unwrap_or("mp4");
    let filename = format!("{id}.{format}");
    let storage_dir = &config.capture.storage_dir;
    let output_path = format!("{storage_dir}/{filename}");

    let mut ffmpeg_args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "x11grab".to_string(),
        "-video_size".to_string(),
        "1920x1080".to_string(),
        "-framerate".to_string(),
        "25".to_string(),
        "-i".to_string(),
        x_display.clone(),
        "-f".to_string(),
        "pulse".to_string(),
        "-i".to_string(),
        "default".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
    ];

    if let Some(secs) = duration_secs {
        ffmpeg_args.push("-t".to_string());
        ffmpeg_args.push(secs.to_string());
    }

    ffmpeg_args.push(output_path.clone());

    let mut ffmpeg = tokio::process::Command::new(&config.capture.ffmpeg_path)
        .args(&ffmpeg_args)
        .env("DISPLAY", &x_display)
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg for display capture")?;

    // --- Step 5: Wait for completion ---
    // Continue reading recorder stdout for RECORDING_STOPPED
    let recorder_stopped = async {
        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim() == "RECORDING_STOPPED" {
                return;
            }
        }
        // stdout closed means process exited
    };

    let result = tokio::select! {
        status = ffmpeg.wait() => {
            let status = status.context("Failed to wait for ffmpeg")?;
            if status.success() {
                tracing::info!(schedule_id = %schedule.id, "ffmpeg capture completed");
                Ok(())
            } else {
                Err(anyhow::anyhow!("ffmpeg exited with status {status}"))
            }
        }
        _ = recorder_stopped => {
            tracing::info!(schedule_id = %schedule.id, "Meeting ended, stopping ffmpeg");
            kill_process("ffmpeg", &mut ffmpeg).await;
            Ok(())
        }
        _ = token.cancelled() => {
            tracing::warn!(schedule_id = %schedule.id, "Browser recording cancelled");
            kill_process("ffmpeg", &mut ffmpeg).await;
            kill_process("recorder", &mut recorder).await;
            Ok(())
        }
    };

    // --- Step 6: Cleanup ---
    kill_process("recorder", &mut recorder).await;
    kill_process("xvfb", &mut xvfb).await;

    // Kill PulseAudio for this session
    let _ = tokio::process::Command::new("pulseaudio")
        .args(["--kill"])
        .env("XDG_RUNTIME_DIR", &pulse_runtime)
        .status()
        .await;

    // Clean up pulse runtime dir
    let _ = tokio::fs::remove_dir_all(&pulse_runtime).await;

    // Propagate ffmpeg errors before finalizing
    result?;

    // --- Step 7: Finalize ---
    finalize_recording(db, config, schedule, &id, &filename, &output_path).await
}
