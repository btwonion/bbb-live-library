use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Category;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/categories/{id}",
            put(update_category).delete(delete_category),
        )
}

async fn list_categories(State(state): State<AppState>) -> Result<Json<Vec<Category>>, AppError> {
    let categories = sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(categories))
}

async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO categories (id, name, description) VALUES (?1, ?2, ?3)")
        .bind(&id)
        .bind(&name)
        .bind(&body.description)
        .execute(&state.db)
        .await?;

    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Json(category)))
}

async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    if body.name.is_none() && body.description.is_none() {
        return Err(AppError::BadRequest(
            "At least one field must be provided".to_string(),
        ));
    }

    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }
    }

    let result = sqlx::query(
        "UPDATE categories SET name = COALESCE(?1, name), description = COALESCE(?2, description) WHERE id = ?3",
    )
    .bind(&body.name)
    .bind(&body.description)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(category))
}

async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = ?1")
        .bind(&id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
