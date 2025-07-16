use chrono::{DateTime, NaiveDateTime, Utc};
use edge_protocol::MeasurementSerieEntry;
use sqlx::SqlitePool;

use crate::data::types::MeasurementRepository;


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
                battery: self.battery as u8,
                lux: self.lux as f32,
                temperature: self.temperature as f32,
                humidity: self.humidity as f32,
            },
        }
    }
}




pub struct SqliteMeasurementRepository {
    pool: SqlitePool
}

impl SqliteMeasurementRepository {
    pub fn new(pool: SqlitePool) -> Self {
        SqliteMeasurementRepository { pool }
    }
}

impl MeasurementRepository for SqliteMeasurementRepository {
    async fn insert(&self, mac: &[u8; 6], entries: Vec<edge_protocol::MeasurementSerieEntry>) -> anyhow::Result<u32> {

        let mut inserted = 0u32;

        for entry in &entries {
            let row = MeasurementSerieEntryRow::from_measurement_serie_entry(mac, entry, 0);

            let mut conn = self.pool.acquire().await?;

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
            .execute(&mut *conn)
            .await?;
        
            inserted += 1;
        }

        Ok(inserted)
    }

    async fn find_by_mac(&self, mac: &[u8; 6]) -> Vec<MeasurementSerieEntry> {
        let mac_ref = mac.as_ref();
        let rows = match sqlx::query_as!(
            MeasurementSerieEntryRow,
            r#"
            SELECT id, mac, timestamp, battery, lux, temperature, humidity
            FROM measurements
            WHERE mac = ?
            "#,
            mac_ref
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows.iter().map(|x| x.to_measurement_serie_entry()).collect(),
            Err(err) => {
                eprintln!("Error occurred: {:?}", err);
                return vec![]
            }
        };

        rows
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
        let found = repo.find_by_mac(&mac).await;
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

        let found = repo.find_by_mac(&mac).await;
        assert!(found.is_empty());
    }
}
