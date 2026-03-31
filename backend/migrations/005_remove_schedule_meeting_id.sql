-- SQLite doesn't support DROP COLUMN before 3.35.0, so recreate the table
CREATE TABLE schedules_new (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    recurrence TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    stream_url TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    room_url TEXT NOT NULL DEFAULT '',
    bot_name TEXT NOT NULL DEFAULT 'Recorder'
);

INSERT INTO schedules_new (id, title, start_time, end_time, recurrence, enabled, created_at, updated_at, stream_url, status, room_url, bot_name)
SELECT id, title, start_time, end_time, recurrence, enabled, created_at, updated_at, stream_url, status, room_url, bot_name
FROM schedules;

DROP TABLE schedules;
ALTER TABLE schedules_new RENAME TO schedules;
