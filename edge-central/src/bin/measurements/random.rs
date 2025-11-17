use std::{pin::Pin, time::Duration};

use chrono::{TimeDelta, Utc};
use edge_protocol::{Measurement, MeasurementSerieEntry};
use futures::{stream, Stream};
use rand::Rng;
use tokio::time::sleep;

use crate::measurements::types::{PeripheralSyncResult, PeripheralSyncResultStreamProvider};

pub struct RandomPeripheralSyncResultStreamProvider {
    pub mac: [u8; 6],
    pub delay: TimeDelta,
}

impl RandomPeripheralSyncResultStreamProvider {
    pub fn new(mac: [u8; 6], delay: TimeDelta) -> Self {
        Self { mac, delay }
    }
}

impl PeripheralSyncResultStreamProvider for RandomPeripheralSyncResultStreamProvider {
    fn stream(&self) -> Pin<Box<dyn Stream<Item = Vec<PeripheralSyncResult>> + Send>> {
        let delay = Duration::from_millis(self.delay.num_milliseconds() as u64);
        let mac = self.mac;
        let stream = stream::unfold((delay, mac), |(delay, mac)| async move {
            let mut measurements = vec![];

            for _ in 0..6 {
                let measurement = random_measurement(&mut rand::rng());
                let serie_entry = MeasurementSerieEntry {
                    timestamp: Utc::now().naive_utc(),
                    measurement,
                };

                measurements.push(serie_entry);
            }

            let result = PeripheralSyncResult {
                address: mac,
                time_drift: TimeDelta::zero(),
                measurements,
            };

            sleep(delay).await;

            Some((vec![result], (delay, mac)))
        });

        Box::pin(stream)
    }
}

fn random_measurement<T: Rng>(rng: &mut T) -> Measurement {
    Measurement {
        battery: (rng.random::<u8>() % 101) as u8,
        lux: (rng.random::<u32>() % 100001) as f32,
        temperature: (rng.random::<u32>() % 46) as f32,
        humidity: (rng.random::<u32>() % 101) as f32,
        soil_pf: (rng.random::<u32>() % 450) as f32,
    }
}
