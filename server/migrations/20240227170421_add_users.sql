CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL CHECK(length(username) <= 30),
    password TEXT NOT NULL
);
