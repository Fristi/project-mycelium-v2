use chrono::NaiveDateTime;
use edge_protocol::MeasurementSerieEntry;
use sqlx::any;


pub trait MeasurementRepository {
    async fn insert(&self, mac: &[u8; 6], entries: Vec<MeasurementSerieEntry>) -> anyhow::Result<u64>;
    async fn find_by_mac(&self, mac: &[u8; 6]) -> anyhow::Result<Vec<MeasurementSerieEntry>>;
}

pub struct EdgeState {
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub auth0_access_token: String,
    pub auth0_refresh_token: String,
    pub auth0_expires_at: NaiveDateTime
}

pub trait EdgeStateRepository {
    async fn get(&self, ) -> anyhow::Result<Option<EdgeState>>;
    async fn set(&self, state: &EdgeState) -> anyhow::Result<u64>;
}