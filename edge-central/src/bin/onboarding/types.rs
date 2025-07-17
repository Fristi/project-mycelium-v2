use crate::data::types::EdgeState;

pub trait Onboarding {
    async fn process() -> anyhow::Result<EdgeState>;
}
