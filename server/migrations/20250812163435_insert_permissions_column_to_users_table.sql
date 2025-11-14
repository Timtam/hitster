ALTER TABLE users ADD COLUMN permissions INTEGER NOT NULL DEFAULT 0;
UPDATE users SET permissions = 0;
