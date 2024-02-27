CREATE TABLE users (
    ID INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL CHECK(length(username) <= 30),
    password_hash TEXT NOT NULL
);
