CREATE TABLE IF NOT EXISTS synthetic_generations (
    scan_id TEXT PRIMARY KEY NOT NULL,
    generation INTEGER NOT NULL CHECK (generation >= 1)
) STRICT;

PRAGMA user_version = 14;
