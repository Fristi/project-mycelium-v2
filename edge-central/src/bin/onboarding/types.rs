use crate::data::types::EdgeState;

pub trait Onboarding {
    async fn process(&self) -> anyhow::Result<EdgeState>;
}
