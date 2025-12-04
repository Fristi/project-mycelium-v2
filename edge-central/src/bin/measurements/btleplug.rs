use std::pin::Pin;
use std::sync::Arc;

use anyhow::anyhow;
use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use chrono::{DateTime, Utc};
use edge_protocol::*;
use futures::Stream;
use tokio::time::{sleep, Duration};
use tracing::info;
use uuid::Uuid;

use crate::measurements::types::{PeripheralSyncResult, PeripheralSyncResultStreamProvider};

const CURRENT_TIME_SERVICE: Uuid = uuid_from_u16(CURRENT_TIME_SERVICE_UUID);
const CURRENT_TIME_CHAR: Uuid = uuid_from_u16(CURRENT_TIME_CHARACTERISTIC_UUID);
const MEASUREMENT_SERVICE: Uuid = uuid_from_u16(MEASUREMENT_SERVICE_UUID_16);
const MEASUREMENT_CHAR: Uuid = uuid_from_u16(MEASUREMENT_CHARACTERISTIC_UUID_16);
const ADDRESS_SERVICE: Uuid = uuid_from_u16(ADDRESS_SERVICE_UUID_16);
const ADDRESS_CHAR: Uuid = uuid_from_u16(ADDRESS_CHARACTERISTIC_UUID_16);

pub struct BtleplugPeripheralSyncResultStreamProvider {
    adapter: Arc<Adapter>,
}

impl BtleplugPeripheralSyncResultStreamProvider {
    pub async fn new() -> anyhow::Result<Self> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .nth(0)
            .ok_or(anyhow!("No adapter found"))?;

        Ok(BtleplugPeripheralSyncResultStreamProvider {
            adapter: Arc::new(adapter),
        })
    }
}

impl PeripheralSyncResultStreamProvider for BtleplugPeripheralSyncResultStreamProvider {
    fn stream(self: Box<Self>) -> Pin<Box<dyn Stream<Item = Vec<PeripheralSyncResult>>>> {
        let adapter = self.adapter.clone();
        let stream = futures::stream::unfold(adapter, |adapter| async {
            if let Err(err) = adapter
                .start_scan(ScanFilter {
                    services: vec![CURRENT_TIME_SERVICE],
                })
                .await {
                tracing::error!(?err, "Btleplug error occurred");
                return None
            };

            

            let peripherals = adapter.peripherals().await.ok()?;
            let mut results = vec![];

            tracing::info!("Found {} peripherals", peripherals.len());

            for peripheral in peripherals {
                let now = Utc::now();
                match sync(peripheral, now).await {
                    Err(err) => tracing::warn!(?err, "Sync error occurred"),
                    Ok(result) => results.push(result)
                }
            }

            sleep(Duration::from_secs(1)).await;

            if let Err(err) = adapter.stop_scan().await {
                tracing::error!(?err, "Btleplug error occurred");
                return None
            };

            Some((results, adapter))
        });

        Box::pin(stream)
    }
}

async fn sync(peripheral: Peripheral, now: DateTime<Utc>) -> anyhow::Result<PeripheralSyncResult> {
    async fn find_characteristic_or_disconnect(
        peripheral: &Peripheral,
        service: Uuid,
        characteristic: Uuid,
    ) -> anyhow::Result<Characteristic> {
        let services = peripheral.services();
        let service = match services.iter().find(|s| s.uuid == service) {
            Some(s) => s,
            None => {
                peripheral.disconnect().await?;
                return Err(anyhow!("Device does not have {} service", service));
            }
        };

        info!("Found device {:?}", peripheral.address());

        let characteristic = match service
            .characteristics
            .iter()
            .find(|c| c.uuid == characteristic)
        {
            Some(c) => c,
            None => {
                peripheral.disconnect().await?;
                return Err(anyhow!(
                    "Device does not have {} characteristic",
                    characteristic
                ));
            }
        };

        Ok(characteristic.clone())
    }

    if !peripheral.is_connected().await? {
        peripheral.connect().await?;
    }

    peripheral.discover_services().await?;

    let address_char =
        find_characteristic_or_disconnect(&peripheral, ADDRESS_SERVICE, ADDRESS_CHAR).await?;
    let data = peripheral.read(&address_char).await?;
    let address: [u8; 6] = data
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("Address data is not 6 bytes"))?;

    let current_time_char =
        find_characteristic_or_disconnect(&peripheral, CURRENT_TIME_SERVICE, CURRENT_TIME_CHAR)
            .await?;
    let data = peripheral.read(&current_time_char).await?;
    let bytes = data.as_slice();
    let current_time = CurrentTime::from_bytes(bytes);
    let datetime = current_time.to_naivedatetime();

    let duration = now.naive_utc() - datetime;
    let ct = CurrentTime::from_naivedatetime(now.naive_utc());
    let bytes = ct.to_bytes();
    peripheral
        .write(&current_time_char, &bytes, WriteType::WithoutResponse)
        .await?;

    let measurement_char =
        find_characteristic_or_disconnect(&peripheral, MEASUREMENT_SERVICE, MEASUREMENT_CHAR)
            .await?;
    let measurement_data = peripheral.read(&measurement_char).await?;

    if measurement_data.len() != 198 {
        peripheral.disconnect().await?;
        return Err(anyhow!("Measurement data is not 198 bytes"));
    }

    let mut measurements = Vec::<MeasurementSerieEntry>::new();

    for i in 0..6 {
        let start = i * 33;
        let end = start + 33;
        let segment = &measurement_data[start..end];

        match MeasurementSerieEntry::from_tlv(segment) {
            Ok(entry) => measurements.push(entry),
            Err(err) => tracing::warn!("Error decoding measurement entry {:?}", err),
        }
    }

    Ok(PeripheralSyncResult {
        address: address,
        time_drift: duration,
        measurements: measurements,
    })
}