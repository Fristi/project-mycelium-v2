pub mod measurements;
pub mod data;

use std::{str::FromStr, sync::Arc};

use anyhow::Ok;
use chrono::Duration;
use futures::{stream, StreamExt};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};

use crate::{data::{sqlite::SqliteMeasurementRepository, types::MeasurementRepository}, measurements::{btleplug::BtleplugPeripheralSyncResultStreamProvider, types::PeripheralSyncResultStreamProvider}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = SqliteConnectOptions::from_str("sqlite://mycelium.db")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .read_only(false);

    // use in a pool
    let pool = SqlitePool::connect_with(opts).await?;

    sqlx::migrate!().run(&pool).await;

    let mac = [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa];
    let repo = Arc::new(SqliteMeasurementRepository::new(pool));
    let provider = measurements::random::RandomPeripheralSyncResultStreamProvider { 
        mac, 
        delay: Duration::milliseconds(2000)
    };

    let stream = provider.stream().flat_map(stream::iter);


    stream.take(3).for_each(|item| async {
        match repo.insert(&mac, item.measurements).await {
            Err(err) => eprintln!("Error occurred while inserting: {:?}", err),
            _ => ()
        };
        ()
    }).await;


    let results = repo.find_by_mac(&mac).await;

    println!("Found results: {}", results.len());

    for result in results {
        println!("Entry: {:?} - {:?} lux", result.timestamp, result.measurement.lux);
    }

    Ok(())
}