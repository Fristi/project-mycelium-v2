use std::time::Duration;

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

impl PeripheralSyncResultStreamProvider for RandomPeripheralSyncResultStreamProvider {
    fn stream(&self) -> impl Stream<Item = Vec<PeripheralSyncResult>> {
        stream::unfold((), async move |_| {
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
                address: self.mac,
                time_drift: TimeDelta::zero(),
                measurements,
            };

            sleep(Duration::from_millis(self.delay.num_milliseconds() as u64)).await;

            Some((vec![result], ()))
        })
    }
}

fn random_measurement<T: Rng>(rng: &mut T) -> Measurement {
    Measurement {
        battery: (rng.random::<u8>() % 101) as u8,
        lux: (rng.random::<u32>() % 100001) as f32,
        temperature: (rng.random::<u32>() % 46) as f32,
        humidity: (rng.random::<u32>() % 101) as f32,
    }
}
