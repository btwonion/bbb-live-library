-- Add new source types for URL import and public BBB import.
-- SQLite doesn't support ALTER CHECK, so we recreate the table.

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
    bbb_meeting_id TEXT,
    schedule_id TEXT REFERENCES schedules(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO recordings_new SELECT * FROM recordings;

DROP TABLE recordings;

ALTER TABLE recordings_new RENAME TO recordings;
