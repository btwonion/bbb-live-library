mod api;
mod bbb;
mod capture;
mod config;
mod db;
mod error;
mod models;

use std::net::SocketAddr;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;
use tokio_util::sync::CancellationToken;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let config = config::load_config("config.toml")?;
    tracing::info!("Loaded configuration");

    // Ensure storage directories exist
    let storage_dir = &config.capture.storage_dir;
    std::fs::create_dir_all(format!("{storage_dir}/thumbs"))?;
    tracing::info!("Storage directory ready: {storage_dir}");

    let pool = db::init_db(&config.database.url).await?;
    tracing::info!("Database initialized");

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    // Cancellation infrastructure
    let token = CancellationToken::new();

    // Spawn background BBB import task if configured
    if let Some(interval_secs) = config.bbb.import_interval_secs {
        let import_db = state.db.clone();
        let import_config = config.clone();
        let import_token = token.clone();
        let interval = std::time::Duration::from_secs(interval_secs);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = import_token.cancelled() => {
                        tracing::info!("BBB import loop shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        tracing::info!("Running scheduled BBB import");
                        match bbb::importer::run_bbb_import(&import_db, &import_config).await {
                            Ok(result) => {
                                tracing::info!(
                                    "BBB import complete: {} imported, {} skipped, {} errors",
                                    result.imported,
                                    result.skipped,
                                    result.errors.len()
                                );
                            }
                            Err(err) => {
                                tracing::error!("BBB import failed: {err:#}");
                            }
                        }
                    }
                }
            }
        });
        tracing::info!("Background BBB import enabled (interval: {interval_secs}s)");
    }

    // Spawn background capture scheduler
    {
        let scheduler_db = state.db.clone();
        let scheduler_config = config.clone();
        let scheduler_token = token.clone();
        tokio::spawn(capture::scheduler::run_scheduler(
            scheduler_db,
            scheduler_config,
            scheduler_token,
        ));
        tracing::info!("Background capture scheduler started");
    }

    // SPA static file serving
    let frontend_dir = config
        .server
        .frontend_dir
        .as_deref()
        .unwrap_or("frontend/dist");
    let spa = ServeDir::new(frontend_dir)
        .not_found_service(ServeFile::new(format!("{frontend_dir}/index.html")));

    let app = Router::new()
        .route("/api/health", get(health))
        .merge(api::router())
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(spa);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Graceful shutdown
    let shutdown_token = token.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_token))
        .await?;

    tracing::info!("Server stopped, cancelling background tasks...");
    token.cancel();

    // Give background tasks a moment to clean up
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    tracing::info!("Shutdown complete");
    Ok(())
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    }
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, shutting down..."),
        _ = terminate => tracing::info!("Received SIGTERM, shutting down..."),
        _ = token.cancelled() => {},
    }
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
