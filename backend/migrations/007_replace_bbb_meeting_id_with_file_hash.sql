-- Replace bbb_meeting_id with file_hash for universal duplicate detection.

CREATE TABLE recordings_new (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    duration_seconds INTEGER,
    file_size_bytes INTEGER,
    format TEXT NOT NULL DEFAULT 'webm',
    source TEXT NOT NULL CHECK (source IN ('live_capture', 'bbb_import', 'url_import', 'bbb_public')),
    file_hash TEXT,
    schedule_id TEXT REFERENCES schedules(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO recordings_new (id, title, description, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, schedule_id, created_at, updated_at)
    SELECT id, title, description, file_path, thumbnail_path, duration_seconds, file_size_bytes, format, source, schedule_id, created_at, updated_at
    FROM recordings;

DROP TABLE recordings;

ALTER TABLE recordings_new RENAME TO recordings;
