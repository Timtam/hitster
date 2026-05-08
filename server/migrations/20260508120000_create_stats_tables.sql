CREATE TABLE hit_stats (
    -- hit id, UUID4 string
    hit_id TEXT UNIQUE PRIMARY KEY,
    -- number of rounds in which this hit was awarded to the guessing player
    correct_guesses BIGINT NOT NULL DEFAULT 0,
    -- number of times this hit was skipped via the skip endpoint
    skips BIGINT NOT NULL DEFAULT 0,
    -- number of times the turn player got a token after this hit was revealed (confirm true)
    tokens_earned BIGINT NOT NULL DEFAULT 0,
    -- number of times this hit was pulled from the stack and revealed for guessing
    reveals BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (hit_id) REFERENCES hits (id) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE user_stats (
    -- user id, UUID4 string
    user_id TEXT UNIQUE PRIMARY KEY,
    -- number of games this user joined that were started
    games_played BIGINT NOT NULL DEFAULT 0,
    -- number of games this user won (reached the goal hit count)
    games_won BIGINT NOT NULL DEFAULT 0,
    -- number of rounds in which this user was awarded the hit
    hits_guessed_correctly BIGINT NOT NULL DEFAULT 0,
    -- number of tokens this user earned via confirm-true (excludes start tokens)
    tokens_earned BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
) WITHOUT ROWID;
