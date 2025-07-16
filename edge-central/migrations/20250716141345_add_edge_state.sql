-- Add migration script here
CREATE TABLE edge_state (
    id INTEGER PRIMARY KEY,
    wifi_ssid TEXT NOT NULL,
    wifi_password TEXT NOT NULL,
    auth0_access_token TEXT NOT NULL,
    auth0_refresh_token TEXT NOT NULL,
    auth0_expires_at DATETIME NOT NULL
);
