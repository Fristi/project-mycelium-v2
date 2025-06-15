use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType, ValueNotification};
use btleplug::platform::{Adapter, Manager};
use uuid::Uuid;
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::time::{sleep, Duration};
use std::alloc::System;
use std::error::Error;
use std::time::SystemTime;
use current_time::CurrentTime;

const CURRENT_TIME_SERVICE_UUID: Uuid = uuid_from_u16(0x1805);
const CURRENT_TIME_CHAR_UUID: Uuid = uuid_from_u16(0x2a2b);

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
            // if let Ok(Some(props)) = peripheral.properties().await {
            //     println!("Checking device: {:?}", props.local_name);
            // } else {
            //     continue;
            // }

            println!("Connecting to device");

            if !peripheral.is_connected().await? {
                if let Err(e) = peripheral.connect().await {
                    println!("Failed to connect: {}", e);
                    continue;
                }
            }

            println!("Connected to device");

            peripheral.discover_services().await?;

            let has_current_time_service = peripheral.services().iter().any(|s| {
                s.uuid == CURRENT_TIME_SERVICE_UUID
            });

            if !has_current_time_service {
                peripheral.disconnect().await?;
                println!("Disconnected device");
                continue;
            }

            println!("Found device with Current Time Service!");

            let characteristics = peripheral.characteristics();
            let characteristic = characteristics.iter().find(|c| {
                c.uuid == CURRENT_TIME_CHAR_UUID
            });

            if let Some(charac) = characteristic {
                // let now = Utc::now().naive_utc();
                // // Read current time from device
                // match peripheral.read(charac).await {
                //     Ok(data) => {
                //         let bytes = data.try_into().expect("Unable to convert");
                //         let current_time = CurrentTime::from_bytes(&bytes);
                //         let datetime = current_time.to_naivedatetime();                    

                //         let duration = now - datetime;
                //         println!("Time drift: {:?}", duration);
                //         println!("Here: {:?}, Device: {:?}", now, datetime);
                //     },
                //     Err(e) => println!("Failed to read time: {}", e),
                // }

                let now = Utc::now();
                let ts = now.timestamp();

                let ct = CurrentTime::from_naivedatetime(Utc::now().naive_utc());

                let bytes = ct.to_bytes();
                match peripheral.write(charac, &bytes, WriteType::WithoutResponse).await {
                    Ok(_) => println!("Updated device time to: {}", now),
                    Err(e) => println!("Failed to write time: {}", e),
                }
            }

            peripheral.disconnect().await?;
        }

        println!("Waiting before rescanning...");
        sleep(Duration::from_secs(10)).await;
    }
}
