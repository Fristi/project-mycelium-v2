use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType, ValueNotification};
use btleplug::platform::{Adapter, Manager};
use uuid::Uuid;
use chrono::{Local, Timelike, Datelike};
use tokio::time::{sleep, Duration};
use std::error::Error;
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
                // Read current time from device
                match peripheral.read(charac).await {
                    Ok(data) => {
                        let bytes = &data.try_into().expect("Unable to convert");
                        let current_time = CurrentTime::from_bytes(bytes);

                        println!("Time on device {:?}", current_time);
                    },
                    Err(e) => println!("Failed to read time: {}", e),
                }

                // Write current system time to the device
                // let now = Local::now();
                // let mut buffer = vec![];
                // buffer.extend_from_slice(&now.year().to_le_bytes()); // Year
                // buffer.push(now.month() as u8);
                // buffer.push(now.day() as u8);
                // buffer.push(now.hour() as u8);
                // buffer.push(now.minute() as u8);
                // buffer.push(now.second() as u8);
                // buffer.push(0); // Day of week (0 = unknown)
                // buffer.push(0); // Fractions256
                // buffer.push(0); // Adjust reason

                // match peripheral.write(charac, &buffer, WriteType::WithResponse).await {
                //     Ok(_) => println!("Updated device time to: {}", now),
                //     Err(e) => println!("Failed to write time: {}", e),
                // }
            }

            peripheral.disconnect().await?;
        }

        println!("Waiting before rescanning...");
        sleep(Duration::from_secs(10)).await;
    }
}
