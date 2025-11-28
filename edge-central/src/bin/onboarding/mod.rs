use crate::cfg::{AppConfig, OnboardingStrategy};
use crate::onboarding::local::LocalOnboarding;
use crate::onboarding::types::Onboarding;

pub mod local;
pub mod types;

pub async fn make_onboarding(cfg: &AppConfig) -> anyhow::Result<Box<dyn Onboarding>> {
    match cfg.onboarding_strategy {
        OnboardingStrategy::Ble => todo!(),
        OnboardingStrategy::Local => {
            let onboarding = LocalOnboarding::new(cfg.auth0.clone(), cfg.wifi.clone());
            anyhow::Ok(Box::new(onboarding))
        }
    }
}