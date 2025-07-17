use serde::{de::DeserializeOwned, Deserialize};

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
    let payload = form
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
        .into_bytes();

    let resp = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("content-length", payload.len().to_string())
        .body(payload)
        .send()
        .await?;

    let res = resp.json::<T>().await?;

    Ok(res)
}

const DEFAULT_AUTH0_DOMAIN: &str = "mycelium-greens.eu.auth0.com";
const DEFAULT_AUTH0_CLIENT_ID: &str = "i9p7v3jAPo8z6mwiuCt6IB5dGNAG1xaz";

pub async fn refresh_token(refresh_token: &str) -> anyhow::Result<TokenResult> {
    let domain = option_env!("AUTH0_DOMAIN").unwrap_or(DEFAULT_AUTH0_DOMAIN);
    let client_id = option_env!("AUTH0_CLIENT_ID").unwrap_or(DEFAULT_AUTH0_CLIENT_ID);
    let client_secret = option_env!("AUTH0_CLIENT_SECRET")
        .unwrap_or("zp-7XzX4rP-ihysBSPoF2fXLfQRAxv2WnJEw-dp4f2LEa_rN8T2gU4fU-OqxWg4I");
    post_form(
        &format!("https://{}/oauth/token", domain),
        [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ],
    )
    .await
}

pub async fn poll_token(device_code: &str) -> anyhow::Result<TokenResult> {
    let domain = option_env!("AUTH0_DOMAIN").unwrap_or(DEFAULT_AUTH0_DOMAIN);
    let client_id = option_env!("AUTH0_CLIENT_ID").unwrap_or(DEFAULT_AUTH0_CLIENT_ID);
    post_form(
        &format!("https://{}/oauth/token", domain),
        [
            ("client_id", client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ],
    )
    .await
}

pub async fn request_device_code() -> anyhow::Result<DeviceCodeResponse> {
    let domain = option_env!("AUTH0_DOMAIN").unwrap_or(DEFAULT_AUTH0_DOMAIN);
    let client_id = option_env!("AUTH0_CLIENT_ID").unwrap_or(DEFAULT_AUTH0_CLIENT_ID);
    let scope = option_env!("AUTH0_SCOPE").unwrap_or("offline_access");
    let audience = option_env!("AUTH0_AUDIENCE").unwrap_or("https://mycelium.co");

    post_form(
        &format!("https://{}/oauth/device/code", domain),
        [
            ("client_id", client_id),
            ("scope", scope),
            ("audience", audience),
        ],
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_request_device_code_invalid_domain() {
        // Set an invalid domain to force a failure
        env::set_var("AUTH0_DOMAIN", "invalid.domain");
        let result = request_device_code().await;
        assert!(result.is_err(), "Expected error for invalid domain");
    }

    #[tokio::test]
    async fn test_poll_token_invalid_device_code() {
        // Use a clearly invalid device code
        let result = poll_token("invalid_device_code").await;
        assert!(result.is_err(), "Expected error for invalid device code");
    }

    #[tokio::test]
    async fn test_refresh_token_invalid_token() {
        // Use a clearly invalid refresh token
        let result = refresh_token("invalid_refresh_token").await;
        assert!(result.is_ok());
    }
}
