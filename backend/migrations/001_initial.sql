CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    meeting_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    recurrence TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recordings (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    duration_seconds INTEGER,
    file_size_bytes INTEGER,
    format TEXT NOT NULL DEFAULT 'webm',
    source TEXT NOT NULL CHECK (source IN ('live_capture', 'bbb_import')),
    bbb_meeting_id TEXT,
    schedule_id TEXT REFERENCES schedules(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recording_categories (
    recording_id TEXT NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
    category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (recording_id, category_id)
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recording_tags (
    recording_id TEXT NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (recording_id, tag_id)
);
