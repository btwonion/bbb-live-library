use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Tag;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/tags", get(list_tags).post(create_tag))
}

async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(tags))
}

async fn create_tag(
    State(state): State<AppState>,
    Json(body): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO tags (id, name) VALUES (?1, ?2)")
        .bind(&id)
        .bind(&name)
        .execute(&state.db)
        .await?;

    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Json(tag)))
}
