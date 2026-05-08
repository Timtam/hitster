-- number of rounds in which this user successfully intercepted (stole) a hit while not being the turn player
ALTER TABLE user_stats ADD COLUMN hits_stolen_successfully BIGINT NOT NULL DEFAULT 0;
-- number of rounds in which this user attempted to intercept but didn't get the hit
ALTER TABLE user_stats ADD COLUMN hits_steal_attempts_failed BIGINT NOT NULL DEFAULT 0;
-- reset all user stats: previously persisted rows only had a subset of these counters,
-- which would make the new derived percentages misleading
DELETE FROM user_stats;
