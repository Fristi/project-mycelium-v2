use async_trait::async_trait;

use crate::data::types::EdgeState;

#[async_trait]
pub trait Onboarding {
    async fn process(&self) -> anyhow::Result<EdgeState>;
}
