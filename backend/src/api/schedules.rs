use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::api::{PaginatedResponse, PaginationParams};
use crate::error::AppError;
use crate::models::Schedule;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub title: String,
    pub stream_url: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub meeting_id: Option<String>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub title: Option<String>,
    pub stream_url: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub meeting_id: Option<String>,
    pub recurrence: Option<String>,
    pub enabled: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/schedules", get(list_schedules).post(create_schedule))
        .route(
            "/api/schedules/{id}",
            get(get_schedule)
                .put(update_schedule)
                .delete(delete_schedule),
        )
}

async fn list_schedules(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Schedule>>, AppError> {
    let (offset, limit) = params.offset_limit();

    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM schedules")
            .fetch_one(&state.db)
            .await?;

    let schedules = sqlx::query_as::<_, Schedule>(
        "SELECT * FROM schedules ORDER BY start_time DESC LIMIT ?1 OFFSET ?2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PaginatedResponse {
        data: schedules,
        total: total.0,
        page: params.page.unwrap_or(1).max(1),
        per_page: limit,
    }))
}

async fn create_schedule(
    State(state): State<AppState>,
    Json(body): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<Schedule>), AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let meeting_id = body.meeting_id.unwrap_or_default();

    let schedule = sqlx::query_as::<_, Schedule>(
        "INSERT INTO schedules (id, title, stream_url, meeting_id, start_time, end_time, recurrence, enabled, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 'pending', datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(&body.title)
    .bind(&body.stream_url)
    .bind(&meeting_id)
    .bind(&body.start_time)
    .bind(&body.end_time)
    .bind(&body.recurrence)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(schedule)))
}

async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Schedule>, AppError> {
    let schedule = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Schedule not found".to_string()))?;

    Ok(Json(schedule))
}

async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateScheduleRequest>,
) -> Result<Json<Schedule>, AppError> {
    let result = sqlx::query(
        "UPDATE schedules SET
            title = COALESCE(?1, title),
            stream_url = COALESCE(?2, stream_url),
            start_time = COALESCE(?3, start_time),
            end_time = COALESCE(?4, end_time),
            meeting_id = COALESCE(?5, meeting_id),
            recurrence = COALESCE(?6, recurrence),
            enabled = COALESCE(?7, enabled),
            updated_at = datetime('now')
         WHERE id = ?8",
    )
    .bind(&body.title)
    .bind(&body.stream_url)
    .bind(&body.start_time)
    .bind(&body.end_time)
    .bind(&body.meeting_id)
    .bind(&body.recurrence)
    .bind(body.enabled)
    .bind(&id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Schedule not found".to_string()));
    }

    let schedule = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(schedule))
}

async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Check if currently recording
    let schedule = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Schedule not found".to_string()))?;

    if schedule.status == "recording" {
        return Err(AppError::Conflict(
            "Cannot delete a schedule that is currently recording".to_string(),
        ));
    }

    sqlx::query("DELETE FROM schedules WHERE id = ?1")
        .bind(&id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
