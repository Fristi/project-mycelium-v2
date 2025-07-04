use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::time::{sleep, Duration};
use std::error::Error;
use edge_protocol::{CurrentTime, Measurement};
use anyhow::*;

const CURRENT_TIME_SERVICE_UUID: Uuid = uuid_from_u16(0x1805);
const CURRENT_TIME_CHAR_UUID: Uuid = uuid_from_u16(0x2a2b);

struct PeripheralSyncResult {
    time_drift: Duration,
    measurements: Vec<Measurement>,
}

trait PeripheralSync {
    async fn sync(&self, time: DateTime<Utc>) -> Result<PeripheralSyncResult>;
}

struct BlePeripheralSync {
    peripheral: Peripheral,
}

impl BlePeripheralSync {
    fn new(peripheral: Peripheral) -> Self {
        Self { peripheral }
    }
}

impl PeripheralSync for BlePeripheralSync {

    async fn sync(&self, now: DateTime<Utc>) -> Result<PeripheralSyncResult> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }

        self.peripheral.discover_services().await?;

        let has_current_time_service = self.peripheral.services().iter().any(|s| {
            s.uuid == CURRENT_TIME_SERVICE_UUID
        });

        if !has_current_time_service {
            self.peripheral.disconnect().await?;
            println!("Disconnected device");
            return Err(anyhow!("Device does not have Current Time Service"));
        }

        println!("Found device with Current Time Service!");

        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics.iter().find(|c| {
            c.uuid == CURRENT_TIME_CHAR_UUID
        });

        let charac = match characteristic {
            Some(c) => c,
            None => {
                self.peripheral.disconnect().await?;
                println!("Disconnected device");
                return Err(anyhow!("Device does not have Current Time Characteristic"));
            }
        };

        // Read current time from device
        let data = self.peripheral.read(charac).await?;
        let bytes = data.as_slice();
        let current_time = CurrentTime::from_bytes(bytes);
        let datetime = current_time.to_naivedatetime();                    

        let duration = now.naive_utc() - datetime;
        let ct = CurrentTime::from_naivedatetime(now.naive_utc());
        let bytes = ct.to_bytes();
        self.peripheral.write(charac, &bytes, WriteType::WithResponse).await?;

        Ok(PeripheralSyncResult {
            time_drift: Duration::from_nanos(duration.num_nanoseconds().unwrap_or(0) as u64),
            measurements: vec![],
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters
        .into_iter()
        .nth(0)
        .ok_or("No Bluetooth adapters found")?;

    loop {
        println!("Scanning for BLE devices with Current Time Service...");
        adapter.start_scan(ScanFilter { services: vec![CURRENT_TIME_SERVICE_UUID] }).await?;
        sleep(Duration::from_secs(4)).await;

        let peripherals = adapter.peripherals().await?;
        for peripheral in peripherals {
            let sync = BlePeripheralSync::new(peripheral);
            let result = sync.sync(Utc::now()).await?;
            println!("Time drift: {:?}", result.time_drift);
        }

        println!("Waiting before rescanning...");
        sleep(Duration::from_secs(10)).await;
    }
}
