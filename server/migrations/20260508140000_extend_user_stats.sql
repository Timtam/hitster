-- number of rounds in which this user guessed but didn't get the hit awarded (as turn player)
ALTER TABLE user_stats ADD COLUMN hits_guessed_wrong BIGINT NOT NULL DEFAULT 0;
-- number of times this user was the turn player during a confirm-false (didn't get a token)
ALTER TABLE user_stats ADD COLUMN tokens_missed BIGINT NOT NULL DEFAULT 0;
