use crate::status::{Status, StatusSummary};

pub struct NoopStatus;

impl NoopStatus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Status for NoopStatus {
    fn show(&mut self, _summary: &StatusSummary) -> anyhow::Result<()> {
        Ok(())
    }
}
