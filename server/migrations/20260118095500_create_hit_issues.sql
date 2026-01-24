CREATE TABLE hit_issues (
    -- issue id, UUID4 string
    id TEXT UNIQUE PRIMARY KEY,
    -- hit id, UUID4 string
    hit_id TEXT NOT NULL,
    -- issue type (extendable)
    type TEXT NOT NULL,
    -- issue description or user message
    message TEXT NOT NULL,
    -- date of creation
    created_at TEXT NOT NULL,
    -- date of last modification
    last_modified TEXT NOT NULL,
    FOREIGN KEY (hit_id) REFERENCES hits (id) ON DELETE CASCADE
) WITHOUT ROWID;
