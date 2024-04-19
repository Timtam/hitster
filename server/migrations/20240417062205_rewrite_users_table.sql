DROP TABLE users;

CREATE TABLE users (
    id TEXT UNIQUE PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    tokens TEXT
) WITHOUT ROWID;
