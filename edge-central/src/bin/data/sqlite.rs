use chrono::{DateTime, NaiveDateTime, Utc};
use edge_protocol::MeasurementSerieEntry;
use sqlx::SqlitePool;

use crate::data::types::{EdgeState, EdgeStateRepository, MeasurementRepository};


#[derive(Debug, sqlx::FromRow)]
pub struct MeasurementSerieEntryRow {
    pub id: i64,
    pub mac: Vec<u8>,
    pub timestamp: NaiveDateTime,
    pub battery: i64,
    pub lux: f64,
    pub temperature: f64,
    pub humidity: f64
}

impl MeasurementSerieEntryRow {
    pub fn from_measurement_serie_entry(
        mac: &[u8; 6],
        entry: &edge_protocol::MeasurementSerieEntry,
        id: i64,
    ) -> Self {
        MeasurementSerieEntryRow {
            id,
            mac: mac.to_vec(),
            timestamp: entry.timestamp,
            battery: entry.measurement.battery as i64,
            lux: entry.measurement.lux as f64,
            temperature: entry.measurement.temperature as f64,
            humidity: entry.measurement.humidity as f64,
        }
    }

    pub fn to_measurement_serie_entry(&self) -> edge_protocol::MeasurementSerieEntry {
        edge_protocol::MeasurementSerieEntry {
            timestamp: self.timestamp,
            measurement: edge_protocol::Measurement {
                battery: self.battery.try_into().unwrap_or(100),
                lux: self.lux as f32,
                temperature: self.temperature as f32,
                humidity: self.humidity as f32,
            },
        }
    }
}


#[derive(Debug, sqlx::FromRow)]
pub struct EdgeStateRow {
    pub id: i64,
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub auth0_access_token: String,
    pub auth0_refresh_token: String,
    pub auth0_expires_at: NaiveDateTime,
}

impl EdgeStateRow {
    pub fn from_edge_state(state: &crate::data::types::EdgeState) -> Self {
        EdgeStateRow {
            // We only store one edge state, so the identifier is hard-coded to 1
            id: 1,
            wifi_ssid: state.wifi_ssid.clone(),
            wifi_password: state.wifi_password.clone(),
            auth0_access_token: state.auth0_access_token.clone(),
            auth0_refresh_token: state.auth0_refresh_token.clone(),
            auth0_expires_at: state.auth0_expires_at,
        }
    }

    pub fn to_edge_state(&self) -> crate::data::types::EdgeState {
        crate::data::types::EdgeState {
            wifi_ssid: self.wifi_ssid.clone(),
            wifi_password: self.wifi_password.clone(),
            auth0_access_token: self.auth0_access_token.clone(),
            auth0_refresh_token: self.auth0_refresh_token.clone(),
            auth0_expires_at: self.auth0_expires_at,
        }
    }
}


pub struct SqliteEdgeStateRepository {
    pool: SqlitePool
}

impl SqliteEdgeStateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EdgeStateRepository for SqliteEdgeStateRepository {
    async fn get(&self) -> anyhow::Result<Option<EdgeState>> {
        let row = sqlx::query_as!(
            EdgeStateRow,
            r#"
            SELECT id, wifi_ssid, wifi_password, auth0_access_token, auth0_refresh_token, auth0_expires_at
            FROM edge_state
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.to_edge_state()))
    }

    async fn set(&self, state: &EdgeState) -> anyhow::Result<u64> {
        let row = EdgeStateRow::from_edge_state(&state);

        let res = sqlx::query!(
            r#"
            INSERT INTO edge_state (id, wifi_ssid, wifi_password, auth0_access_token, auth0_refresh_token, auth0_expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                wifi_ssid = excluded.wifi_ssid,
                wifi_password = excluded.wifi_password,
                auth0_access_token = excluded.auth0_access_token,
                auth0_refresh_token = excluded.auth0_refresh_token,
                auth0_expires_at = excluded.auth0_expires_at
            "#,
            row.id,
            row.wifi_ssid,
            row.wifi_password,
            row.auth0_access_token,
            row.auth0_refresh_token,
            row.auth0_expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }
}


pub struct SqliteMeasurementRepository {
    pool: SqlitePool
}

impl SqliteMeasurementRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl MeasurementRepository for SqliteMeasurementRepository {
    async fn insert(&self, mac: &[u8; 6], entries: Vec<edge_protocol::MeasurementSerieEntry>) -> anyhow::Result<u64> {

        let mut inserted = 0u64;
        let mut tx = self.pool.begin().await?;

        for entry in &entries {
            let row = MeasurementSerieEntryRow::from_measurement_serie_entry(mac, entry, 0);
            let res = sqlx::query!(
                r#"
                INSERT INTO measurements (mac, timestamp, battery, lux, temperature, humidity)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                row.mac,
                row.timestamp,
                row.battery,
                row.lux,
                row.temperature,
                row.humidity
            )
            .execute(&mut *tx)
            .await?;
        
            inserted += res.rows_affected();
        }

        tx.commit().await?;

        Ok(inserted)
    }

    async fn find_by_mac(&self, mac: &[u8; 6]) -> anyhow::Result<Vec<MeasurementSerieEntry>> {
        let mac_ref = mac.as_ref();
        let rows = sqlx::query_as!(
            MeasurementSerieEntryRow,
            r#"
            SELECT id, mac, timestamp, battery, lux, temperature, humidity
            FROM measurements
            WHERE mac = ?
            "#,
            mac_ref
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|x| x.to_measurement_serie_entry()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{sqlite::SqlitePoolOptions, Executor};
    use edge_protocol::{MeasurementSerieEntry, Measurement};
    use chrono::{Utc, TimeZone};

    #[tokio::test]
    async fn test_insert_and_find_by_mac() {
        // Create in-memory SQLite database
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("Failed to create pool");

        // Run migrations instead of creating the table directly
        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

        let repo = super::SqliteMeasurementRepository::new(pool.clone());

        let mac = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

        let entry = MeasurementSerieEntry {
            timestamp: Utc.timestamp_opt(1_700_000_000, 0).unwrap().naive_utc(),
            measurement: Measurement {
                battery: 30,
                lux: 123.4,
                temperature: 22.5,
                humidity: 55.0,
            },
        };

        let entries = vec![entry.clone()];

        // Insert entries
        let inserted = repo.insert(&mac, entries.clone()).await.expect("Insert failed");
        assert_eq!(inserted, 1);

        // Find by mac
        let found = repo.find_by_mac(&mac).await.expect("Unable to find by mac");
        assert_eq!(found.len(), 1);

        let found_entry = &found[0];
        assert_eq!(found_entry.timestamp.timestamp(), entry.timestamp.timestamp());
        assert_eq!(found_entry.measurement.battery, entry.measurement.battery);
        assert_eq!(found_entry.measurement.lux, entry.measurement.lux);
        assert_eq!(found_entry.measurement.temperature, entry.measurement.temperature);
        assert_eq!(found_entry.measurement.humidity, entry.measurement.humidity);
    }

    #[tokio::test]
    async fn test_find_by_mac_empty() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("Failed to create pool");

        // Run migrations instead of creating the table directly
        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

        let repo = super::SqliteMeasurementRepository::new(pool.clone());

        let mac = [0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];

        let found = repo.find_by_mac(&mac).await.expect("Unable to find by mac");
        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn test_edge_state_get_none() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("Failed to create pool");

        // Run migrations to create the edge_state table
        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

        let repo = SqliteEdgeStateRepository::new(pool.clone());

        // There should be no record yet
        let result = repo.get().await.expect("Unable to get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_edge_state_set_and_get_some() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("Failed to create pool");

        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

        let repo = SqliteEdgeStateRepository::new(pool.clone());

        let state = EdgeState {
            wifi_ssid: "ssid1".to_string(),
            wifi_password: "pass1".to_string(),
            auth0_access_token: "token1".to_string(),
            auth0_refresh_token: "refresh1".to_string(),
            auth0_expires_at: NaiveDateTime::from_timestamp_opt(1_700_000_000, 0).unwrap(),
        };

        // Set the state
        let affected = repo.set(&state).await.expect("Unable to set state");
        assert_eq!(affected, 1);

        // Get the state
        let result = repo.get().await.expect("Unable to get state");
        assert!(result.is_some());
        let loaded = result.unwrap();
        assert_eq!(loaded.wifi_ssid, state.wifi_ssid);
        assert_eq!(loaded.wifi_password, state.wifi_password);
        assert_eq!(loaded.auth0_access_token, state.auth0_access_token);
        assert_eq!(loaded.auth0_refresh_token, state.auth0_refresh_token);
        assert_eq!(loaded.auth0_expires_at, state.auth0_expires_at);
    }

    #[tokio::test]
    async fn test_edge_state_set_overwrites() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("Failed to create pool");

        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

        let repo = SqliteEdgeStateRepository::new(pool.clone());

        let state1 = EdgeState {
            wifi_ssid: "ssid1".to_string(),
            wifi_password: "pass1".to_string(),
            auth0_access_token: "token1".to_string(),
            auth0_refresh_token: "refresh1".to_string(),
            auth0_expires_at: NaiveDateTime::from_timestamp_opt(1_700_000_000, 0).unwrap(),
        };

        let state2 = EdgeState {
            wifi_ssid: "ssid1".to_string(),
            wifi_password: "pass2".to_string(),
            auth0_access_token: "token2".to_string(),
            auth0_refresh_token: "refresh2".to_string(),
            auth0_expires_at: NaiveDateTime::from_timestamp_opt(1_800_000_000, 0).unwrap(),
        };

        // Set the first state
        let affected1 = repo.set(&state1).await.expect("Unable to set state");
        assert_eq!(affected1, 1);

        // Set the second state (should update)
        let affected2 = repo.set(&state2).await.expect("Unable to set state");
        // Depending on SQLite, this may be 1 or 0 if nothing changed, but at least it should not error

        // Get the state and check it's the updated one
        let result = repo.get().await.expect("Unable to get state");
        assert!(result.is_some());
        let loaded = result.unwrap();
        assert_eq!(loaded.wifi_ssid, state2.wifi_ssid);
        assert_eq!(loaded.wifi_password, state2.wifi_password);
        assert_eq!(loaded.auth0_access_token, state2.auth0_access_token);
        assert_eq!(loaded.auth0_refresh_token, state2.auth0_refresh_token);
        assert_eq!(loaded.auth0_expires_at, state2.auth0_expires_at);
    }
}
