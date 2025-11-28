use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};
use futures::Stream;
use futures::stream::{self, StreamExt};
use uuid::Uuid;
use trouble_host::prelude::*;
use edge_protocol::*;
use crate::measurements::types::{PeripheralSyncResult, PeripheralSyncResultStreamProvider};

pub struct TroublePeripheralSyncResultStreamProvider<C>
where
    C: Controller + Send + Sync + 'static,
{
    controller: Arc<C>,
}

impl<C> TroublePeripheralSyncResultStreamProvider<C>
where
    C: Controller + Send + Sync + 'static,
{
    pub fn new(controller: C) -> Self {
        Self {
            controller: Arc::new(controller),
        }
    }

    async fn sync_peripheral(
        &self,
        peripheral_addr: [u8; 6],
    ) -> anyhow::Result<PeripheralSyncResult> {
        let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
        let mut resources: HostResources<DefaultPacketPool, 1, 3> = HostResources::new();
        let stack = trouble_host::new(&*self.controller, &mut resources).set_random_address(address);
        let Host { mut central, mut runner, .. } = stack.build();

        let target: Address = Address::random(peripheral_addr);

        let config = ConnectConfig {
            connect_params: Default::default(),
            scan_config: ScanConfig {
                filter_accept_list: &[(target.kind, &target.addr)],
                ..Default::default()
            },
        };

        let conn = join(runner.run(), async {
            info!("Scanning and connecting...");
            central.connect(&config).await.unwrap()
        })
        .await
        .1;

        let client = GattClient::<_, DefaultPacketPool, 10>::new(&stack, &conn)
            .await
            .unwrap();

        // --- Address ---
        let addr_service = client
            .services_by_uuid(&Uuid::new_short(ADDRESS_SERVICE_UUID_16))
            .await?
            .first()
            .unwrap()
            .clone();
        let addr_char: Characteristic<u8> =
            client.characteristic_by_uuid(&addr_service, &Uuid::new_short(ADDRESS_CHAR_UUID_16))
                .await?;
        let mut addr_data = [0u8; 6];
        client.read_characteristic(&addr_char, &mut addr_data).await?;
        let address: [u8; 6] = addr_data;

        // --- Current Time ---
        let now = Utc::now();
        let time_service = client
            .services_by_uuid(&Uuid::new_short(CURRENT_TIME_SERVICE_UUID))
            .await?
            .first()
            .unwrap()
            .clone();
        let time_char: Characteristic<u8> =
            client.characteristic_by_uuid(&time_service, &Uuid::new_short(CURRENT_TIME_CHAR_UUID))
                .await?;
        let mut current_time_data = [0u8; 10]; // adjust size to match CurrentTime
        client.read_characteristic(&time_char, &mut current_time_data).await?;
        let current_time = CurrentTime::from_bytes(&current_time_data);
        let datetime = current_time.to_naivedatetime();
        let duration = now.naive_utc() - datetime;

        let ct = CurrentTime::from_naivedatetime(now.naive_utc());
        let bytes = ct.to_bytes();
        client.write_characteristic(&time_char, &bytes).await?;

        // --- Measurement ---
        let meas_service = client
            .services_by_uuid(&Uuid::new_short(MEASUREMENT_SERVICE_UUID_16))
            .await?
            .first()
            .unwrap()
            .clone();
        let meas_char: Characteristic<u8> =
            client.characteristic_by_uuid(&meas_service, &Uuid::new_short(MEASUREMENT_CHAR_UUID_16))
                .await?;
        let mut measurement_data = [0u8; 198];
        client.read_characteristic(&meas_char, &mut measurement_data).await?;

        let mut measurements = vec![];
        for i in 0..6 {
            let start = i * 33;
            let end = start + 33;
            match MeasurementSerieEntry::from_tlv(&measurement_data[start..end]) {
                Ok(entry) => measurements.push(entry),
                Err(err) => tracing::warn!("Error decoding measurement entry {:?}", err),
            }
        }

        Ok(PeripheralSyncResult {
            address,
            time_drift: duration,
            measurements,
        })
    }
}

impl<C> PeripheralSyncResultStreamProvider for TroublePeripheralSyncResultStreamProvider<C>
where
    C: Controller + Send + Sync + 'static,
{
    fn stream(&self) -> Pin<Box<dyn Stream<Item = Vec<PeripheralSyncResult>> + Send>> {
        let provider = self.clone();
        let stream = stream::unfold((), move |_| {
            let provider = provider.clone();
            async move {
                // TODO: Replace with real scanning list
                let peripheral_addrs = vec![[0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]];

                let mut results = vec![];
                for addr in peripheral_addrs {
                    match provider.sync_peripheral(addr).await {
                        Ok(res) => results.push(res),
                        Err(err) => tracing::warn!("Sync error: {:?}", err),
                    }
                }

                Timer::after(Duration::from_secs(1)).await;
                Some((results, ()))
            }
        });

        Box::pin(stream)
    }
}
