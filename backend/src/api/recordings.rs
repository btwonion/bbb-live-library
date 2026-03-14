use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::api::{PaginatedResponse, PaginationParams};
use crate::error::AppError;
use crate::models::{Category, Recording, Tag};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RecordingListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub source: Option<String>,
    pub category_id: Option<String>,
    pub tag_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordingRequest {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RecordingDetail {
    #[serde(flatten)]
    pub recording: Recording,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/recordings", get(list_recordings))
        .route(
            "/api/recordings/{id}",
            get(get_recording)
                .post(update_recording)
                .delete(delete_recording),
        )
        .route("/api/recordings/{id}/categories", post(assign_categories))
        .route("/api/recordings/{id}/tags", post(assign_tags))
}

async fn list_recordings(
    State(state): State<AppState>,
    Query(params): Query<RecordingListParams>,
) -> Result<Json<PaginatedResponse<Recording>>, AppError> {
    let pagination = PaginationParams {
        page: params.page,
        per_page: params.per_page,
    };
    let (offset, limit) = pagination.offset_limit();

    let search_pattern = params.search.as_ref().map(|s| format!("%{s}%"));

    // Build dynamic SQL with sequential ? placeholders
    let mut count_sql = String::from("SELECT COUNT(DISTINCT r.id) FROM recordings r");
    let mut query_sql = String::from("SELECT DISTINCT r.* FROM recordings r");
    let mut where_clauses: Vec<String> = Vec::new();
    let mut joins = String::new();

    if params.category_id.is_some() {
        joins.push_str(" JOIN recording_categories rc ON r.id = rc.recording_id");
        where_clauses.push("rc.category_id = ?".to_string());
    }
    if params.tag_id.is_some() {
        joins.push_str(" JOIN recording_tags rt ON r.id = rt.recording_id");
        where_clauses.push("rt.tag_id = ?".to_string());
    }
    if search_pattern.is_some() {
        where_clauses.push("(r.title LIKE ? OR r.description LIKE ?)".to_string());
    }
    if params.source.is_some() {
        where_clauses.push("r.source = ?".to_string());
    }

    count_sql.push_str(&joins);
    query_sql.push_str(&joins);

    if !where_clauses.is_empty() {
        let where_str = format!(" WHERE {}", where_clauses.join(" AND "));
        count_sql.push_str(&where_str);
        query_sql.push_str(&where_str);
    }

    query_sql.push_str(" ORDER BY r.created_at DESC LIMIT ? OFFSET ?");

    // Bind filter params in the same order for both queries
    macro_rules! bind_filters {
        ($q:expr) => {{
            let mut q = $q;
            if let Some(ref cid) = params.category_id {
                q = q.bind(cid);
            }
            if let Some(ref tid) = params.tag_id {
                q = q.bind(tid);
            }
            if let Some(ref pat) = search_pattern {
                q = q.bind(pat).bind(pat); // bound twice for title + description
            }
            if let Some(ref src) = params.source {
                q = q.bind(src);
            }
            q
        }};
    }

    let total: (i64,) = bind_filters!(sqlx::query_as::<_, (i64,)>(&count_sql))
        .fetch_one(&state.db)
        .await?;

    let recordings: Vec<Recording> = {
        let q = bind_filters!(sqlx::query_as::<_, Recording>(&query_sql));
        q.bind(limit).bind(offset).fetch_all(&state.db).await?
    };

    Ok(Json(PaginatedResponse {
        data: recordings,
        total: total.0,
        page: pagination.page.unwrap_or(1).max(1),
        per_page: limit,
    }))
}

async fn get_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RecordingDetail>, AppError> {
    let recording = sqlx::query_as::<_, Recording>("SELECT * FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    let categories = sqlx::query_as::<_, Category>(
        "SELECT c.* FROM categories c JOIN recording_categories rc ON c.id = rc.category_id WHERE rc.recording_id = ?1",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await?;

    let tags = sqlx::query_as::<_, Tag>(
        "SELECT t.* FROM tags t JOIN recording_tags rt ON t.id = rt.tag_id WHERE rt.recording_id = ?1",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(RecordingDetail {
        recording,
        categories,
        tags,
    }))
}

async fn update_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateRecordingRequest>,
) -> Result<Json<Recording>, AppError> {
    if body.title.is_none() && body.description.is_none() {
        return Err(AppError::BadRequest(
            "At least one field must be provided".to_string(),
        ));
    }

    let result = sqlx::query(
        "UPDATE recordings SET title = COALESCE(?1, title), description = COALESCE(?2, description), updated_at = datetime('now') WHERE id = ?3",
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(&id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Recording not found".to_string()));
    }

    let recording = sqlx::query_as::<_, Recording>("SELECT * FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(recording))
}

async fn delete_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let recording = sqlx::query_as::<_, Recording>("SELECT * FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    sqlx::query("DELETE FROM recordings WHERE id = ?1")
        .bind(&id)
        .execute(&state.db)
        .await?;

    // Best-effort file cleanup
    let storage_dir = &state.config.capture.storage_dir;
    let file_path = format!("{storage_dir}/{}", recording.file_path);
    if let Err(err) = tokio::fs::remove_file(&file_path).await {
        tracing::warn!("Failed to delete recording file {file_path}: {err}");
    }

    if let Some(ref thumb) = recording.thumbnail_path {
        let thumb_path = format!("{storage_dir}/{thumb}");
        if let Err(err) = tokio::fs::remove_file(&thumb_path).await {
            tracing::warn!("Failed to delete thumbnail {thumb_path}: {err}");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn assign_categories(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AssignIdsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify recording exists
    sqlx::query("SELECT id FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM recording_categories WHERE recording_id = ?1")
        .bind(&id)
        .execute(&mut *tx)
        .await?;

    for category_id in &body.ids {
        sqlx::query("INSERT INTO recording_categories (recording_id, category_id) VALUES (?1, ?2)")
            .bind(&id)
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn assign_tags(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AssignIdsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify recording exists
    sqlx::query("SELECT id FROM recordings WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recording not found".to_string()))?;

    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM recording_tags WHERE recording_id = ?1")
        .bind(&id)
        .execute(&mut *tx)
        .await?;

    for tag_id in &body.ids {
        sqlx::query("INSERT INTO recording_tags (recording_id, tag_id) VALUES (?1, ?2)")
            .bind(&id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
