use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct TimezoneResponse {
    pub timezone: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/settings/timezone", get(get_timezone))
}

/// Returns the configured timezone.
async fn get_timezone(State(state): State<AppState>) -> Json<TimezoneResponse> {
    Json(TimezoneResponse {
        timezone: state.config.server.timezone().to_string(),
    })
}
