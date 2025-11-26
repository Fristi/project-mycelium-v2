pub mod i2c;

use anyhow::Result;
use chrono::{Date, DateTime, NaiveDateTime, Utc};
use edge_protocol::MeasurementSerieEntry;

pub trait Status {
    fn show(&mut self, summary: &StatusSummary) -> Result<()>;
}

pub struct StatusSummary {
    pub from: DateTime<Utc>,
    pub till: DateTime<Utc>,
    pub temperature: f32,
    pub humidity: f32,
    pub soil_moisture: f32,
    pub light: f32
}

impl StatusSummary {
    pub fn from_measurements(measurements: &Vec<MeasurementSerieEntry>) -> Option<StatusSummary> {
        if measurements.is_empty() {
            return None;
        }

        // First and last timestamps
        let from_naive: NaiveDateTime = measurements.first()?.timestamp;
        let till_naive: NaiveDateTime = measurements.last()?.timestamp;

        let from = DateTime::<Utc>::from_naive_utc_and_offset(from_naive, Utc);
        let till = DateTime::<Utc>::from_naive_utc_and_offset(till_naive, Utc);

        // Summation
        let mut temp_sum = 0.0;
        let mut humidity_sum = 0.0;
        let mut soil_sum = 0.0;
        let mut light_sum = 0.0;

        for m in measurements {
            temp_sum += m.measurement.temperature;
            humidity_sum += m.measurement.humidity;
            soil_sum += m.measurement.soil_pf;
            light_sum += m.measurement.lux;
        }

        let count = measurements.len() as f32;

        Some(StatusSummary {
            from,
            till,
            temperature: temp_sum / count,
            humidity: humidity_sum / count,
            soil_moisture: soil_sum / count,
            light: light_sum / count,
        })
    }
}