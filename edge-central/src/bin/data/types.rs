use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct EdgeState {
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub auth0_access_token: String,
    pub auth0_refresh_token: String,
    pub auth0_expires_at: NaiveDateTime,
}