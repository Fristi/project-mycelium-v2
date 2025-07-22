use async_trait::async_trait;
use chrono::Utc;
use tokio::time::{sleep, Duration};
use wifi_rs::prelude::Connectivity;

use crate::{
    auth::auth0::{TokenResult, TokenStatus},
    cfg::{Auth0Config, WifiConfig},
    data::types::EdgeState,
    onboarding::types::Onboarding,
};

pub struct LocalOnboarding {
    auth0: Auth0Config,
    wifi: WifiConfig,
}

impl LocalOnboarding {
    pub fn new(auth0: Auth0Config, wifi: WifiConfig) -> Self {
        Self { auth0, wifi }
    }
}

#[async_trait]
impl Onboarding for LocalOnboarding {
    async fn process(&self) -> anyhow::Result<EdgeState> {
        let mut wifi = wifi_rs::WiFi::new(None);
        let wifi_connection = wifi
            .connect(&self.wifi.ssid, &self.wifi.password)
            .unwrap_or(false);

        if !wifi_connection {
            anyhow::bail!("Failed to connect to WiFi with provided credentials");
        }

        sleep(Duration::from_secs(3)).await;

        let device_code = crate::auth::auth0::request_device_code(&self.auth0).await?;

        println!("Verification code: {}", device_code.user_code);
        println!(
            "Verification url: {}",
            device_code.verification_uri_complete
        );

        loop {
            match crate::auth::auth0::poll_token(&self.auth0, device_code.device_code.as_str())
                .await
            {
                Ok(TokenResult::Full {
                    access_token,
                    refresh_token,
                    expires_in,
                }) => {
                    let expires_at = Utc::now() + Duration::from_secs(expires_in);
                    return Ok(EdgeState {
                        wifi_ssid: self.wifi.ssid.clone(),
                        wifi_password: self.wifi.password.clone(),
                        auth0_access_token: access_token,
                        auth0_refresh_token: refresh_token,
                        auth0_expires_at: expires_at.naive_utc(),
                    });
                }
                Ok(TokenResult::AccessToken { .. }) => {
                    println!("Received access token without refresh token, skipping");
                }
                Ok(TokenResult::Error { error }) => match error {
                    TokenStatus::ExpiredToken
                    | TokenStatus::AccessDenied
                    | TokenStatus::InvalidGrant => {
                        anyhow::bail!("Failed with {:?}", error);
                    }
                    TokenStatus::AuthorizationPending | TokenStatus::SlowDown => {
                        println!("Auth0 status: {:?}", error);
                    }
                },
                Err(error) => {
                    anyhow::bail!("Failed with {}", error);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}
