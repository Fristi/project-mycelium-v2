use chrono::Duration;
use edge_protocol::MeasurementSerieEntry;
use futures::Stream;


pub struct PeripheralSyncResult {
    pub address: [u8; 6],
    pub time_drift: Duration,
    pub measurements: Vec<MeasurementSerieEntry>,
}

pub trait PeripheralSyncResultStreamProvider {
    fn stream(&self) -> impl Stream<Item = Vec<PeripheralSyncResult>>;
}