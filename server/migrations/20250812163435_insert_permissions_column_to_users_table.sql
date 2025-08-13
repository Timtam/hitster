ALTER TABLE users ADD COLUMN permissions INTEGER NOT NULL;
UPDATE users SET permissions = 0;
