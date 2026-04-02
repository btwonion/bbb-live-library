mod categories;
mod import;
mod playback;
mod recordings;
mod schedules;
mod stats;

use axum::Router;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PaginationParams {
    pub fn offset_limit(&self) -> (u32, u32) {
        let per_page = self.per_page.unwrap_or(20).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1) * per_page;
        (offset, per_page)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

/// Builds the combined API router for all resource endpoints.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(categories::router())
        .merge(recordings::router())
        .merge(stats::router())
        .merge(import::router())
        .merge(playback::router())
        .merge(schedules::router())
}
