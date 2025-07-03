use heapless::Vec;
use edge_protocol::CurrentTime;
use timeseries::Deviate;





impl Measurement {
    const DEVIATION: Measurement = Measurement {
        battery: 1,
        lux: 50.0,
        temperature: 1.0,
        humidity: 1.0
    };
}

impl Deviate for Measurement {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        return 
            self.battery - other.battery > max_deviation.battery ||
            self.lux - other.lux > max_deviation.lux ||
            self.temperature - other.temperature > max_deviation.temperature ||
            self.humidity - other.humidity > max_deviation.humidity;
    }
}

pub enum DeviceState {
    AwaitingTimeSync,
    Buffering(Vec<Measurement, 10>),
}