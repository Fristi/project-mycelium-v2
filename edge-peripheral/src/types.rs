use chrono::NaiveDateTime;
use timeseries::{Deviate, Series};

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    pub battery: u8,
    pub lux: f32,
    pub temperature: f32,
    pub humidity: f32
}

impl Measurement {
    pub const fn max_deviation() -> Self {
        Self {
            battery: 1,
            lux: 50.0,
            temperature: 1.0,
            humidity: 1.0
        }
    }
}

impl Deviate for Measurement {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        return 
            (self.battery.abs_diff(other.battery)) > max_deviation.battery ||
            (self.lux - other.lux).abs() > max_deviation.lux ||
            (self.temperature - other.temperature).abs() > max_deviation.temperature ||
            (self.humidity - other.humidity).abs() > max_deviation.humidity;
    }
}

#[derive(Debug, Clone)]
pub enum EdgeState {
    WaitingForTimeSync,
    Buffering { buffer: Series<10, NaiveDateTime, Measurement> }
}