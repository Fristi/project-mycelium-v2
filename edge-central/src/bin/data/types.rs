use edge_protocol::MeasurementSerieEntry;


pub trait MeasurementRepository {
    async fn insert(&self, mac: &[u8; 6], entries: Vec<MeasurementSerieEntry>) -> anyhow::Result<u64>;
    async fn find_by_mac(&self, mac: &[u8; 6]) -> anyhow::Result<Vec<MeasurementSerieEntry>>;
}