pub mod auth;
pub mod cfg;
pub mod data;
pub mod measurements;
pub mod onboarding;

use config::Config;
use dotenv::dotenv;
use futures::{stream, StreamExt};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::{str::FromStr, sync::Arc};

use crate::{
    data::sqlite::{MeasurementSerieEntryRow, SqliteMeasurementRepository},
    measurements::{
        btleplug::BtleplugPeripheralSyncResultStreamProvider,
        types::PeripheralSyncResultStreamProvider,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    let app_config = cfg::AppConfig::from_env()?;
    let opts = SqliteConnectOptions::from_str(&app_config.database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .read_only(false);

    // use in a pool
    let pool = SqlitePool::connect_with(opts).await?;

    sqlx::migrate!().run(&pool).await?;

    // let edge_state_repo = SqliteEdgeStateRepository::new(pool);

    // match edge_state_repo.get().await? {
    //     Some(state) => (),
    //     None => todo!()
    // };

    let repo = Arc::new(SqliteMeasurementRepository::new(pool));

    let provider = BtleplugPeripheralSyncResultStreamProvider::new().await?;
    let stream = provider.stream().flat_map(stream::iter).take(1);

    stream
        .for_each(|m| async {
            for measurement in m.measurements {
                let row = MeasurementSerieEntryRow::from_measurement_serie_entry(
                    &[0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa],
                    &measurement,
                    0,
                );

                println!("row: {:?}", row)
            }
        })
        .await;

    Ok(())
}
