use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub recording_count: i64,
    pub total_duration_seconds: i64,
    pub total_size_bytes: i64,
    pub by_source: Vec<SourceCount>,
    pub by_category: Vec<CategoryCount>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SourceCount {
    pub source: String,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CategoryCount {
    pub category_name: String,
    pub count: i64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/stats", get(get_stats))
}

async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, AppError> {
    let totals: (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(duration_seconds), 0), COALESCE(SUM(file_size_bytes), 0) FROM recordings",
    )
    .fetch_one(&state.db)
    .await?;

    let by_source = sqlx::query_as::<_, SourceCount>(
        "SELECT source, COUNT(*) as count FROM recordings GROUP BY source",
    )
    .fetch_all(&state.db)
    .await?;

    let by_category = sqlx::query_as::<_, CategoryCount>(
        "SELECT c.name as category_name, COUNT(rc.recording_id) as count FROM categories c LEFT JOIN recording_categories rc ON c.id = rc.category_id GROUP BY c.id, c.name ORDER BY count DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(StatsResponse {
        recording_count: totals.0,
        total_duration_seconds: totals.1,
        total_size_bytes: totals.2,
        by_source,
        by_category,
    }))
}
