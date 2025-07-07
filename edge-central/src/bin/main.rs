use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::time::{sleep, Duration};
use std::error::Error;
use edge_protocol::*;
use anyhow::{Result, anyhow};

const CURRENT_TIME_SERVICE: Uuid = uuid_from_u16(CURRENT_TIME_SERVICE_UUID);
const CURRENT_TIME_CHAR: Uuid = uuid_from_u16(CURRENT_TIME_CHARACTERISTIC_UUID);
const MEASUREMENT_SERVICE: Uuid = uuid_from_u16(MEASUREMENT_SERVICE_UUID_16);
const MEASUREMENT_CHAR: Uuid = uuid_from_u16(MEASUREMENT_CHARACTERISTIC_UUID_16);
const ADDRESS_SERVICE: Uuid = uuid_from_u16(ADDRESS_SERVICE_UUID_16);
const ADDRESS_CHAR: Uuid = uuid_from_u16(ADDRESS_CHARACTERISTIC_UUID_16);

struct PeripheralSyncResult {
    address: [u8; 6],
    time_drift: Duration,
    measurements: Vec<MeasurementSerieEntry>,
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

        async fn find_characteristic_or_disconnect(peripheral: &Peripheral, service: Uuid, characteristic: Uuid) -> Result<Characteristic> {
            let services = peripheral.services();
            let service = match services.iter().find(|s| s.uuid == service) {
                Some(s) => s,
                None => {
                    peripheral.disconnect().await?;
                    return Err(anyhow!("Device does not have {} service", service));
                }
            };

            let characteristic = match service.characteristics.iter().find(|c| c.uuid == characteristic) {
                Some(c) => c,
                None => {
                    peripheral.disconnect().await?;
                    return Err(anyhow!("Device does not have {} characteristic", characteristic));
                }
            };

            Ok(characteristic.clone())
        }

        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }

        let address_char = find_characteristic_or_disconnect(&self.peripheral, ADDRESS_SERVICE, ADDRESS_CHAR).await?;
        let data = self.peripheral.read(&address_char).await?;
        let address: [u8; 6] = data.as_slice().try_into().map_err(|_| anyhow!("Address data is not 6 bytes"))?;

        let current_time_char = find_characteristic_or_disconnect(&self.peripheral, CURRENT_TIME_SERVICE, CURRENT_TIME_CHAR).await?;
        let data = self.peripheral.read(&current_time_char).await?;
        let bytes = data.as_slice();
        let current_time = CurrentTime::from_bytes(bytes);
        let datetime = current_time.to_naivedatetime();                    

        let duration = now.naive_utc() - datetime;
        let ct = CurrentTime::from_naivedatetime(now.naive_utc());
        let bytes = ct.to_bytes();
        self.peripheral.write(&current_time_char, &bytes, WriteType::WithoutResponse).await?;

        let measurement_char = find_characteristic_or_disconnect(&self.peripheral, MEASUREMENT_SERVICE, MEASUREMENT_CHAR).await?;
        let measurement_data = self.peripheral.read(&measurement_char).await?;
       
        let mut measurements = Vec::<MeasurementSerieEntry>::new();
        
        for i in 0..6 {
            let start = i * 33;
            let end = start + 33;
            let segment = &measurement_data[start..end];
            
            match MeasurementSerieEntry::from_tlv(segment).ok() {
                Some(entry) => measurements.push(entry),
                _ => continue
            }
        }

        Ok(PeripheralSyncResult {
            address: address,
            time_drift: Duration::from_nanos(duration.num_nanoseconds().unwrap_or(0) as u64),
            measurements: measurements
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
            match sync.sync(Utc::now()).await {
                Ok(result) => {
                    println!("Address: {:02X?}", result.address);
                    println!("Time drift: {:?}", result.time_drift);
                    println!("Measurements: {:?}", result.measurements.len());
                    println!("--------");
                }
                Err(err) => {
                    println!("Failed to sync with device: {:?}", err);
                }
            }
        }

        println!("Waiting before rescanning...");
        sleep(Duration::from_secs(10)).await;
    }
}