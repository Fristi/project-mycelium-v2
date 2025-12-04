use std::cell::RefCell;

use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::ControllerCmdSync;
use futures::future::Either;
use futures::future::try_select;
use tracing::error;
use tracing::info;
use trouble_host::prelude::*;
use embassy_time::Duration;

use tokio::{sync::mpsc, task::LocalSet};
use crate::measurements::types::{PeripheralSyncResult, PeripheralSyncResultStreamProvider};
use edge_protocol::*;
use anyhow::*;
use futures::future::try_join;
use futures::future::FutureExt;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

pub struct TroublePeripheralSyncResultStreamProvider {
    rx: mpsc::Receiver<Vec<PeripheralSyncResult>>
}

impl TroublePeripheralSyncResultStreamProvider
{
    async fn retrieve<'a, C : Controller, P : PacketPool, const MAX_SERVICES: usize>(client: &GattClient<'a, C, P, MAX_SERVICES>) -> std::result::Result<PeripheralSyncResult, anyhow::Error> {

        info!("Starting to scan for services...");        

        let services = client.services_by_uuid(&Uuid::new_short(CURRENT_TIME_SERVICE_UUID))
            .await
            .anyhow("Failed to retrieve services")?;

        for service in services {
            info!("Found service {:?}", service)
        }

        // let service: &ServiceHandle = services.first().ok_or_else(|| anyhow!("No service found"))?;

        // info!("Found time service");

        // let _characteristic = client.characteristic_by_uuid::<u8>(&service, &Uuid::new_short(CURRENT_TIME_CHARACTERISTIC_UUID))
        //     .await
        //     .anyhow("Couldn't find charachteristic");

        let result = PeripheralSyncResult { address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00], time_drift: chrono::TimeDelta::seconds(0), measurements: vec![] };

        Ok(result)
    }

    async fn worker<C : Controller + ControllerCmdSync<LeSetScanParams> + 'static>(controller: C, tx: mpsc::Sender<Vec<PeripheralSyncResult>>) -> Result<()> {
        
        let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
        let stack = trouble_host::new(controller, &mut resources);
        let tracker = BdAddrTracker { devices: RefCell::new(Vec::new()) };
        let mut config = ScanConfig::default();
        config.active = false;
        config.phys = PhySet::M1;
        config.interval = Duration::from_secs(1);
        config.window = Duration::from_secs(1);

        loop {
            let Host { central, mut runner, .. } = stack.build();
            let mut scanner = Scanner::new(central);

            info!("Start scanning for devices");

            let run = Box::pin(runner.run_with_handler(&tracker));
            let scan = Box::pin(async {
                let res = scanner.scan(&config).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                res
            });

            match try_select(run, scan).await {
                std::result::Result::Ok(Either::Left(_)) => Err(anyhow!("Error")),
                std::result::Result::Ok(Either::Right(_)) => Ok(()),
                std::result::Result::Err(Either::Left(_err)) => Err(anyhow!("Error occured")),
                std::result::Result::Err(Either::Right(_err)) => Err(anyhow!("Error occured"))
            }?;

            info!("Finished scanning device");

            let Host { mut central, mut runner, .. } = stack.build();
            let mut results = vec![];

            for (addr_kind, addr) in tracker.devices.borrow().clone() {
                let connect_config = ConnectConfig {
                    connect_params: ConnectParams {
                        min_connection_interval: Duration::from_micros(7500),
                        max_connection_interval: Duration::from_micros(7500),
                        max_latency: 500,
                        supervision_timeout: Duration::from_secs(10),
                        ..Default::default()
                    },
                    scan_config: ScanConfig {
                        filter_accept_list: &[(addr_kind, &addr)],
                        ..Default::default()
                    },
                };

                let (_, conn) = try_join(runner.run(), central.connect(&connect_config))
                    .await
                    .anyhow("Failed to connected to BLE device")?;

                let client: GattClient::<_, DefaultPacketPool, 10> = GattClient::new(&stack, &conn)
                    .await
                    .anyhow("Failed to construct GATT client")?;

                let (_, res) = try_join(client.task().map(|x| x.anyhow("Sync failed")), TroublePeripheralSyncResultStreamProvider::retrieve(&client))
                    .await?;
                
                results.push(res);
            }

            tx.send(results).await.anyhow("Push failed")?;
        }
    }

    pub async fn new<C : Controller + ControllerCmdSync<LeSetScanParams> + 'static>(controller: C, ls: LocalSet) -> Self {

        let (tx, rx) = mpsc::channel(32);

        ls.run_until(async move {
            if let Err(err) = tokio::task::spawn_local(TroublePeripheralSyncResultStreamProvider::worker(controller, tx)).await {
                error!("Doesn't compute {:?}", err)
            }
        }).await;

        info!("Succesfully spawned local background task");

        Self { rx }
    }
}


impl PeripheralSyncResultStreamProvider for TroublePeripheralSyncResultStreamProvider
{
    fn stream(self: Box<Self>) -> std::pin::Pin<Box<dyn futures::Stream<Item = Vec<super::types::PeripheralSyncResult>>>> {
        Box::pin(tokio_stream::wrappers::ReceiverStream::new(self.rx))
    }
}

struct BdAddrTracker {
    pub devices: RefCell<Vec<(AddrKind, BdAddr)>>
}

impl EventHandler for BdAddrTracker {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let mut devices = self.devices.borrow_mut();
        devices.clear();
        while let Some(std::result::Result::Ok(report)) = it.next() {
            //TODO: see in report if it matches

            info!("Advertising data for {:?} --> {:02x?}", report.addr, report.data);

            devices.push((report.addr_kind, report.addr));
        }
    }
}

pub trait ResultAny<T, E> {
    fn anyhow(self, ctx: &'static str) -> Result<T, anyhow::Error>;
}

impl<T, E: core::fmt::Debug> ResultAny<T, E> for Result<T, E> {
    fn anyhow(self, ctx: &'static str) -> Result<T, anyhow::Error> {
        self.map_err(|e| anyhow::anyhow!("{}: {:?}", ctx, e))
    }
}

