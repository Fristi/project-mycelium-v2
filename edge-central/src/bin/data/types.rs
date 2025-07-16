use edge_protocol::MeasurementSerieEntry;


pub trait MeasurementRepository {
    async fn insert(&self, mac: &[u8; 6], entries: Vec<MeasurementSerieEntry>) -> anyhow::Result<u32>;
    async fn find_by_mac(&self, mac: &[u8; 6]) -> Vec<MeasurementSerieEntry>;
}