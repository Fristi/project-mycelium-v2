-- Add migration script here
CREATE TABLE measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mac BLOB NOT NULL, -- 6 bytes
    timestamp DATETIME NOT NULL,
    battery INTEGER NOT NULL,
    lux REAL NOT NULL,
    temperature REAL NOT NULL,
    humidity REAL NOT NULL
);
    