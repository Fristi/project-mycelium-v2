use crate::{data::types::EdgeState, onboarding::types::Onboarding};

struct LocalOnboarding {}

impl Onboarding for LocalOnboarding {
    async fn process() -> anyhow::Result<EdgeState> {
        todo!()
    }
}
