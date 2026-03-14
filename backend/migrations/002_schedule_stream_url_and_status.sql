ALTER TABLE schedules ADD COLUMN stream_url TEXT NOT NULL DEFAULT '';
ALTER TABLE schedules ADD COLUMN status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'recording', 'completed', 'missed'));
