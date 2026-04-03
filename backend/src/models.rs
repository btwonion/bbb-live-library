use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub title: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub recurrence: Option<String>,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub stream_url: String,
    pub status: String,
    pub room_url: String,
    pub bot_name: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub duration_seconds: Option<i64>,
    pub file_size_bytes: Option<i64>,
    pub format: String,
    pub source: String,
    pub file_hash: Option<String>,
    pub schedule_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct RecordingCategory {
    pub recording_id: String,
    pub category_id: String,
}
