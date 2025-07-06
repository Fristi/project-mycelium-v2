use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::time::{sleep, Duration};
use std::error::Error;
use edge_protocol::{CurrentTime, Measurement, CURRENT_TIME_CHARACTERISTIC_UUID, CURRENT_TIME_SERVICE_UUID};
use anyhow::*;

// const CURRENT_TIME_SERVICE: Uuid = Uuid::from_bytes(CURRENT_TIME_SERVICE_UUID);
// const CURRENT_TIME_CHAR: Uuid = Uuid::from_bytes(CURRENT_TIME_CHARACTERISTIC_UUID);

const CURRENT_TIME_SERVICE: Uuid = uuid_from_u16(CURRENT_TIME_SERVICE_UUID);
const CURRENT_TIME_CHAR: Uuid = uuid_from_u16(CURRENT_TIME_CHARACTERISTIC_UUID);


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

        println!("Connecting to device...");

        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;

            println!("Connected to device");
        }

        println!("Discovering services...");
        self.peripheral.discover_services().await?;

        println!("Checking for Current Time Service...");
        let services =  self.peripheral.services();
        let current_time_svc = services.iter().find(|s| {
            s.uuid == CURRENT_TIME_SERVICE
        });

        let svc = match current_time_svc {
            Some(s) => s,
            None => {
                self.peripheral.disconnect().await?;
                println!("Disconnected device");
                return Err(anyhow!("Device does not have Current Time Service"));
            }
        };
        println!("Found device with Current Time Service!");



        let characteristic = svc.characteristics.iter().find(|c| {
            c.uuid == CURRENT_TIME_CHAR
        });

        println!("Found characteristic: {:?}", characteristic);

        let charac = match characteristic {
            Some(c) => c,
            None => {
                self.peripheral.disconnect().await?;
                println!("Disconnected device");
                return Err(anyhow!("Device does not have Current Time Characteristic"));
            }
        };

        println!("Reading current time from device...");
        // Read current time from device
        let data = self.peripheral.read(charac).await?;
        println!("Read data: {:?}", data);
        let bytes = data.as_slice();
        let current_time = CurrentTime::from_bytes(bytes);
        let datetime = current_time.to_naivedatetime();                    
        println!("Current time: {:?}", datetime);

        let duration = now.naive_utc() - datetime;
        println!("Time drift: {:?}", duration);

        let ct = CurrentTime::from_naivedatetime(now.naive_utc());
        let bytes = ct.to_bytes();
        self.peripheral.write(charac, &bytes, WriteType::WithoutResponse).await?;
        println!("Wrote current time to device");

        self.peripheral.disconnect().await?;
        println!("Disconnected device");

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
        adapter.start_scan(ScanFilter { services: vec![CURRENT_TIME_SERVICE] }).await?;
        sleep(Duration::from_secs(4)).await;

        let peripherals = adapter.peripherals().await?;
        for peripheral in peripherals {
            let sync = BlePeripheralSync::new(peripheral);
            match sync.sync(Utc::now()).await.ok() {
                Some(result) => {
                    println!("Time drift: {:?}", result.time_drift);
                }
                None => {
                    println!("Failed to sync with device");
                }
            }
        }

        println!("Waiting before rescanning...");
        sleep(Duration::from_secs(10)).await;
    }
}
