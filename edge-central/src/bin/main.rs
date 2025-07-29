pub mod auth;
pub mod cfg;
pub mod data;
pub mod measurements;
pub mod onboarding;

use aliri_reqwest::AccessTokenMiddleware;
use aliri_tokens::{backoff, jitter, sources::{self, oauth2::dto::RefreshTokenCredentialsSource}, ClientId, RefreshToken, TokenLifetimeConfig, TokenWatcher};
use anyhow::Ok;
use chrono::TimeDelta;
use dotenv::dotenv;
use edge_client_backend::{apis::configuration::{Configuration}, models::{StationInsert, StationMeasurement}};
use futures::{stream, StreamExt};
use reqwest::{Client, Request, Url};
use reqwest_middleware::ClientBuilder;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::{str::FromStr, sync::Arc};

use crate::{
    cfg::{AppConfig, OnboardingStrategy, PeripheralSyncMode},
    data::sqlite::{
        SqliteEdgeStateRepository,
    },
    measurements::{
        btleplug::BtleplugPeripheralSyncResultStreamProvider,
        random::RandomPeripheralSyncResultStreamProvider,
        types::{PeripheralSyncResult, PeripheralSyncResultStreamProvider},
    },
    onboarding::{local::LocalOnboarding, types::Onboarding},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    // Install a subscriber that logs to stdout with TRACE level enabled
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE) // allow trace level logs
        .init();

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

    let jitter_source = jitter::NullJitter;
    let refresh_token = _edge_state.auth0_refresh_token.clone();
    let refresh_token_ref = RefreshToken::new(refresh_token).into_boxed_ref();
    let client_id = ClientId::new(app_config.auth0.client_id);
    let token_url = Url::parse(format!("https://{}/oauth/token", &app_config.auth0.domain).as_str())?;
    let credentials = RefreshTokenCredentialsSource { 
        client_id: client_id,
        client_secret: None,
        refresh_token: refresh_token_ref
    };
    let lifetime_config = TokenLifetimeConfig::default();
    let token_source = sources::oauth2::RefreshTokenSource::new(
        Client::default(), 
        token_url, 
        credentials, 
        lifetime_config
    );

    let token_watcher = TokenWatcher::spawn_from_token_source(token_source, jitter_source, backoff::ErrorBackoffConfig::default()).await?;

    let client = ClientBuilder::new(Client::default())
        .with(AccessTokenMiddleware::new(token_watcher).with_predicate(AlwaysMatch))
        .build();

    let configuration: Configuration = Configuration {
        base_path: app_config.backend_url,
        user_agent: None,
        client: client,
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None                
    };

    let provider = make_peripheral_sync_stream_provider(&app_config.peripheral_sync_mode).await?;
    let stream = provider.stream().flat_map(stream::iter);

    stream
        .for_each(|m| async {
            if let Err(err) = sync_measurements(&configuration, m).await {
                tracing::error!("Failed to sync measurements {}", err);
            }
        })
        .await;

    Ok(())
}

async fn sync_measurements(configuration: &Configuration, m: PeripheralSyncResult) -> anyhow::Result<()> {

    let mac = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", m.address[0], m.address[1], m.address[2], m.address[3], m.address[4], m.address[5]);

    let station_insert = StationInsert::new(mac, "Unnamed".to_string());

    let id = edge_client_backend::apis::default_api::add_station(&configuration, station_insert).await?;
    let mut measurements = vec![];

    for measurement in m.measurements {
        measurements.push(StationMeasurement {
            on: measurement.timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            battery_voltage: 0_f64,
            temperature: measurement.measurement.temperature as f64,
            humidity: measurement.measurement.humidity as f64,
            lux: measurement.measurement.lux as f64,
            soil_pf: 0_f64,
            tank_pf: 0_f64
        });
    }

    edge_client_backend::apis::default_api::checkin_station(&configuration, id.to_string().as_str(), Some(measurements)).await?;

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

#[derive(Debug, Clone)]
pub struct AlwaysMatch;

impl std::fmt::Display for AlwaysMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AlwaysMatch")
    }
}

impl predicates_core::reflection::PredicateReflection for AlwaysMatch {}

impl predicates_core::Predicate<Request> for AlwaysMatch {
    fn eval(&self, _variable: &Request) -> bool {
        true
    }
}