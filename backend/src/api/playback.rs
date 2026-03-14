use std::io::SeekFrom;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{
    ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE,
};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::models::Recording;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/recordings/{id}/stream", get(stream_recording))
        .route("/api/recordings/{id}/thumbnail", get(serve_thumbnail))
}

/// Returns the MIME type for a given recording format.
fn content_type_for_format(format: &str) -> &'static str {
    match format {
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "ogg" | "ogv" => "video/ogg",
        "avi" => "video/x-msvideo",
        _ => "application/octet-stream",
    }
}

/// Parses a `Range` header value like `bytes=0-1023` into (start, optional end).
fn parse_range_header(range: &str) -> Option<(u64, Option<u64>)> {
    let bytes_prefix = "bytes=";
    if !range.starts_with(bytes_prefix) {
        return None;
    }
    let range = &range[bytes_prefix.len()..];
    let mut parts = range.splitn(2, '-');
    let start: u64 = parts.next()?.parse().ok()?;
    let end: Option<u64> = parts.next().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            s.parse().ok()
        }
    });
    Some((start, end))
}

/// Streams a recording file with HTTP range request support.
async fn stream_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let recording = sqlx::query_as::<_, Recording>("SELECT * FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    let storage_dir = &state.config.capture.storage_dir;
    let file_path = format!("{storage_dir}/{}", recording.file_path);

    let mut file = tokio::fs::File::open(&file_path).await.map_err(|err| {
        tracing::warn!("Recording file not found on disk: {file_path}: {err}");
        AppError::NotFound("Recording file not found on disk".to_string())
    })?;

    let metadata = file.metadata().await.map_err(|err| {
        tracing::error!("Failed to read file metadata: {err}");
        AppError::Internal(err.into())
    })?;
    let file_size = metadata.len();
    let content_type = content_type_for_format(&recording.format);

    if let Some(range_value) = headers.get(RANGE) {
        let range_str = range_value.to_str().map_err(|_| {
            AppError::BadRequest("Invalid Range header".to_string())
        })?;

        let (start, end) = parse_range_header(range_str)
            .ok_or_else(|| AppError::BadRequest("Malformed Range header".to_string()))?;

        let end = end.unwrap_or(file_size - 1).min(file_size - 1);

        if start > end || start >= file_size {
            return Ok(Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(CONTENT_RANGE, format!("bytes */{file_size}"))
                .body(Body::empty())
                .unwrap());
        }

        let length = end - start + 1;

        file.seek(SeekFrom::Start(start)).await.map_err(|err| {
            tracing::error!("Failed to seek file: {err}");
            AppError::Internal(err.into())
        })?;

        let limited = file.take(length);
        let stream = ReaderStream::new(limited);

        Ok(Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, length)
            .header(CONTENT_RANGE, format!("bytes {start}-{end}/{file_size}"))
            .header(ACCEPT_RANGES, "bytes")
            .body(Body::from_stream(stream))
            .unwrap())
    } else {
        let stream = ReaderStream::new(file);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, file_size)
            .header(ACCEPT_RANGES, "bytes")
            .body(Body::from_stream(stream))
            .unwrap())
    }
}

/// Serves a recording's thumbnail image.
async fn serve_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let recording = sqlx::query_as::<_, Recording>("SELECT * FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    let thumbnail_path = recording
        .thumbnail_path
        .ok_or_else(|| AppError::NotFound("No thumbnail available".to_string()))?;

    let storage_dir = &state.config.capture.storage_dir;
    let full_path = format!("{storage_dir}/{thumbnail_path}");

    let file = tokio::fs::File::open(&full_path).await.map_err(|err| {
        tracing::warn!("Thumbnail file not found on disk: {full_path}: {err}");
        AppError::NotFound("Thumbnail file not found on disk".to_string())
    })?;

    let metadata = file.metadata().await.map_err(|err| {
        tracing::error!("Failed to read thumbnail metadata: {err}");
        AppError::Internal(err.into())
    })?;

    let stream = ReaderStream::new(file);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "image/jpeg")
        .header(CONTENT_LENGTH, metadata.len())
        .body(Body::from_stream(stream))
        .unwrap())
}
