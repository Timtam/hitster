-- Convert users.id from hyphenated TEXT (e.g. 'd6516cda-31a0-48fa-8000-6557913902e4')
-- to a 16-byte BLOB so it matches sqlx::Uuid's default Sqlite encoding used by the rest
-- of the codebase. Without this, FK references to users.id from BLOB-keyed tables (e.g.
-- user_stats) silently fail because BLOB != TEXT in SQLite, even when the bytes encode
-- the same UUID.
--
-- The WHERE typeof(id) = 'text' guard makes the migration idempotent and a no-op for
-- rows already stored as BLOB.
UPDATE users SET id = unhex(replace(id, '-', '')) WHERE typeof(id) = 'text';
