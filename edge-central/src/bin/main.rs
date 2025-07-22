pub mod auth;
pub mod cfg;
pub mod data;
pub mod measurements;
pub mod onboarding;

use anyhow::Ok;
use chrono::TimeDelta;
use dotenv::dotenv;
use futures::{stream, StreamExt};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::{str::FromStr, sync::Arc};

use crate::{
    cfg::{AppConfig, OnboardingStrategy, PeripheralSyncMode},
    data::sqlite::{
        MeasurementSerieEntryRow, SqliteEdgeStateRepository,
    },
    measurements::{
        btleplug::BtleplugPeripheralSyncResultStreamProvider,
        random::RandomPeripheralSyncResultStreamProvider,
        types::PeripheralSyncResultStreamProvider,
    },
    onboarding::{local::LocalOnboarding, types::Onboarding},
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
    let pool = Arc::new(SqlitePool::connect_with(opts).await?);

    sqlx::migrate!().run(&*pool).await?;

    let edge_state_repo = SqliteEdgeStateRepository::new(pool.clone());
    let _edge_state = match edge_state_repo.get_state().await? {
        Some(state) => state,
        None => {
            let onboarding = make_onboarding(&app_config).await?;
            let edge_state = onboarding.process().await?;
            edge_state_repo.set_state(&edge_state).await?;
            edge_state
        }
    };

    let provider = make_peripheral_sync_stream_provider(&app_config.peripheral_sync_mode).await?;
    let stream = provider.stream().flat_map(stream::iter);

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

async fn make_peripheral_sync_stream_provider(
    mode: &PeripheralSyncMode,
) -> anyhow::Result<Box<dyn PeripheralSyncResultStreamProvider>> {
    match mode {
        PeripheralSyncMode::Ble => {
            let provider = BtleplugPeripheralSyncResultStreamProvider::new().await?;
            Ok(Box::new(provider))
        }
        PeripheralSyncMode::Random => {
            let provider = RandomPeripheralSyncResultStreamProvider::new(
                [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa],
                TimeDelta::seconds(2),
            );

            Ok(Box::new(provider))
        }
    }
}

async fn make_onboarding(cfg: &AppConfig) -> anyhow::Result<Box<dyn Onboarding>> {
    match cfg.onboarding_strategy {
        OnboardingStrategy::Ble => todo!(),
        OnboardingStrategy::Local => {
            let onboarding = LocalOnboarding::new(cfg.auth0.clone(), cfg.wifi.clone());
            Ok(Box::new(onboarding))
        }
    }
}
