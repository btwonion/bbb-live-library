use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Deserializer};

use chrono::DateTime;

use crate::api::PaginatedResponse;
use crate::error::AppError;
use crate::models::Schedule;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ScheduleListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub filter: Option<String>,
}

impl ScheduleListParams {
    fn offset_limit(&self) -> (u32, u32) {
        let per_page = self.per_page.unwrap_or(20).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1) * per_page;
        (offset, per_page)
    }
}

/// Deserializes a doubly-optional field: absent → `None`, explicit null → `Some(None)`, value → `Some(Some(v))`.
fn deserialize_optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

/// Normalizes a datetime string to `YYYY-MM-DD HH:MM:SS` format.
///
/// Accepts ISO 8601 (e.g. `2026-04-03T14:00:00.000Z`) or the target format itself.
fn normalize_datetime(input: &str) -> Result<String, AppError> {
    // Try ISO 8601 / RFC 3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string());
    }

    // Try the target format (already normalized)
    if chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").is_ok() {
        return Ok(input.to_string());
    }

    Err(AppError::BadRequest(format!(
        "Invalid datetime format: {input}. Expected ISO 8601 or YYYY-MM-DD HH:MM:SS"
    )))
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub title: String,
    pub stream_url: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub recurrence: Option<String>,
    pub room_url: Option<String>,
    pub bot_name: Option<String>,
    pub start_offset_secs: Option<i64>,
    pub end_offset_secs: Option<i64>,
    pub category_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub title: Option<String>,
    pub stream_url: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub recurrence: Option<String>,
    pub enabled: Option<bool>,
    pub room_url: Option<String>,
    pub bot_name: Option<String>,
    pub start_offset_secs: Option<i64>,
    pub end_offset_secs: Option<i64>,
    #[serde(deserialize_with = "deserialize_optional_nullable")]
    #[serde(default)]
    pub category_id: Option<Option<String>>,
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
    Query(params): Query<ScheduleListParams>,
) -> Result<Json<PaginatedResponse<Schedule>>, AppError> {
    let (offset, limit) = params.offset_limit();

    let filter_clause = match params.filter.as_deref() {
        Some("active") => " WHERE (status IN ('pending', 'recording')) OR (recurrence IS NOT NULL AND enabled = 1)",
        Some("past") => " WHERE (status IN ('completed', 'missed')) AND (recurrence IS NULL OR enabled = 0)",
        _ => "",
    };

    let count_sql = format!("SELECT COUNT(*) FROM schedules{filter_clause}");
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await?;

    let list_sql = format!(
        "SELECT * FROM schedules{filter_clause} ORDER BY start_time DESC LIMIT ?1 OFFSET ?2"
    );
    let schedules = sqlx::query_as::<_, Schedule>(&list_sql)
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
    let stream_url = body.stream_url.unwrap_or_default();
    let room_url = body.room_url.unwrap_or_default();

    if stream_url.is_empty() && room_url.is_empty() {
        return Err(AppError::BadRequest(
            "Either stream_url or room_url must be provided".to_string(),
        ));
    }

    let start_time = normalize_datetime(&body.start_time)?;
    let end_time = body
        .end_time
        .as_deref()
        .map(normalize_datetime)
        .transpose()?;

    let id = uuid::Uuid::new_v4().to_string();
    let bot_name = body.bot_name.unwrap_or_else(|| "Recorder".to_string());
    let start_offset = body.start_offset_secs.unwrap_or(30);
    let end_offset = body.end_offset_secs.unwrap_or(30);

    let schedule = sqlx::query_as::<_, Schedule>(
        "INSERT INTO schedules (id, title, stream_url, start_time, end_time, recurrence, enabled, status, room_url, bot_name, start_offset_secs, end_offset_secs, category_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 'pending', ?7, ?8, ?9, ?10, ?11, datetime('now'), datetime('now'))
         RETURNING *",
    )
    .bind(&id)
    .bind(&body.title)
    .bind(&stream_url)
    .bind(&start_time)
    .bind(&end_time)
    .bind(&body.recurrence)
    .bind(&room_url)
    .bind(&bot_name)
    .bind(start_offset)
    .bind(end_offset)
    .bind(&body.category_id)
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
    let start_time = body
        .start_time
        .as_deref()
        .map(normalize_datetime)
        .transpose()?;
    let end_time = body
        .end_time
        .as_deref()
        .map(normalize_datetime)
        .transpose()?;

    // Resolve category_id: None = don't change, Some(None) = clear, Some(Some(v)) = set
    let category_id_update: Option<Option<String>> = body.category_id;

    let result = sqlx::query(
        "UPDATE schedules SET
            title = COALESCE(?1, title),
            stream_url = COALESCE(?2, stream_url),
            start_time = COALESCE(?3, start_time),
            end_time = COALESCE(?4, end_time),
            recurrence = COALESCE(?5, recurrence),
            enabled = COALESCE(?6, enabled),
            room_url = COALESCE(?7, room_url),
            bot_name = COALESCE(?8, bot_name),
            start_offset_secs = COALESCE(?9, start_offset_secs),
            end_offset_secs = COALESCE(?10, end_offset_secs),
            category_id = CASE WHEN ?11 THEN ?12 ELSE category_id END,
            updated_at = datetime('now')
         WHERE id = ?13",
    )
    .bind(&body.title)
    .bind(&body.stream_url)
    .bind(&start_time)
    .bind(&end_time)
    .bind(&body.recurrence)
    .bind(body.enabled)
    .bind(&body.room_url)
    .bind(&body.bot_name)
    .bind(body.start_offset_secs)
    .bind(body.end_offset_secs)
    .bind(category_id_update.is_some()) // ?11: whether to update category_id
    .bind(category_id_update.as_ref().and_then(|v| v.as_ref())) // ?12: new value (may be null)
    .bind(&id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Schedule not found".to_string()));
    }

    // Reset status to pending if time/cron fields were changed (not while recording)
    let time_changed = body.start_time.is_some() || body.end_time.is_some() || body.recurrence.is_some();
    if time_changed {
        sqlx::query(
            "UPDATE schedules SET status = 'pending', updated_at = datetime('now') WHERE id = ?1 AND status IN ('completed', 'missed')",
        )
        .bind(&id)
        .execute(&state.db)
        .await?;
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
