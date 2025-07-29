use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize};

use crate::cfg::Auth0Config;

#[derive(Deserialize, Debug)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u32,
    pub interval: u64,
}

#[derive(Deserialize, Debug)]
pub enum TokenStatus {
    #[serde(rename = "authorization_pending")]
    AuthorizationPending,
    #[serde(rename = "slow_down")]
    SlowDown,
    #[serde(rename = "expired_token")]
    ExpiredToken,
    #[serde(rename = "access_denied")]
    AccessDenied,
    #[serde(rename = "invalid_grant")]
    InvalidGrant,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TokenResult {
    Full {
        access_token: String,
        refresh_token: String,
        expires_in: u64,
    },
    AccessToken {
        access_token: String,
        expires_in: u64,
    },
    Error {
        error: TokenStatus,
    },
}

async fn post_form<T, const N: usize>(url: &str, form: [(&str, &str); N]) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let payload: HashMap<&str, &str> = form.iter().cloned().collect();

    let resp = reqwest::Client::new()
        .post(url)
        .form(&payload)
        .send()
        .await?;

    let res = resp.json::<T>().await?;

    Ok(res)
}

pub async fn poll_token(cfg: &Auth0Config, device_code: &str) -> anyhow::Result<TokenResult> {
    post_form(
        &format!("https://{}/oauth/token", &cfg.domain),
        [
            ("client_id", &cfg.client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ],
    )
    .await
}

pub async fn request_device_code(cfg: &Auth0Config) -> anyhow::Result<DeviceCodeResponse> {
    post_form(
        &format!("https://{}/oauth/device/code", &cfg.domain),
        [
            ("client_id", &cfg.client_id),
            ("scope", &cfg.scope),
            ("audience", &cfg.audience),
        ],
    )
    .await
}
