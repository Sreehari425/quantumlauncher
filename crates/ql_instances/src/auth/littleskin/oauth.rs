use ql_reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CLIENT_ID: &str = "1151";


#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] ql_reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LittleSkin error: {0}")]
    LittleSkin(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenResponse {
    pub token_type: String,
    pub expires_in: u64,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
}

/// Step 1: Generate the authorization URL for the user to visit
// Authorization Code flow is deprecated for device flow use-case. If needed, reintroduce REDIRECT_URI.
pub fn authorization_url(scope: &str) -> String {
    format!(
        "https://littleskin.cn/oauth/authorize?client_id={}&response_type=code&scope={}",
        CLIENT_ID, scope
    )
}

/// Step 2: Exchange the authorization code for tokens
pub async fn exchange_code_for_token(
    client: &Client,
    code: &str,
) -> Result<TokenResponse, OAuthError> {
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        // ("client_secret", CLIENT_SECRET),
        // ("redirect_uri", REDIRECT_URI),
        ("code", code),
    ];
    let resp = client
        .post("https://littleskin.cn/oauth/token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::LittleSkin(err_text));
    }
    let token: TokenResponse = resp.json().await?;
    Ok(token)
}

/// Step 3: Refresh the access token using the refresh token
pub async fn refresh_token(
    client: &Client,
    refresh_token: &str,
) -> Result<TokenResponse, OAuthError> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
        // ("client_secret", CLIENT_SECRET),
    ];
    let resp = client
        .post("https://littleskin.cn/oauth/token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::LittleSkin(err_text));
    }
    let token: TokenResponse = resp.json().await?;
    Ok(token)
}

/// Step 4: Get user info using the access token
pub async fn get_user_info(
    client: &Client,
    access_token: &str,
) -> Result<UserInfo, OAuthError> {
    let resp = client
        .get("https://littleskin.cn/api/user")
        .header("Accept", "application/json")
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::LittleSkin(err_text));
    }
    let user: UserInfo = resp.json().await?;
    Ok(user)
}

/// Device Code Flow structs and functions for LittleSkin
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceTokenResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Step 1: Request device code
pub async fn request_device_code(
    client: &Client,
    client_id: &str,
    scope: &str,
) -> Result<DeviceCodeResponse, OAuthError> {
    // Build body exactly as LittleSkin expects: "client_id={client_id}&\nscope={scope}"
    let encoded_scope = urlencoding::encode(scope);
    let body = format!("client_id={}&scope={}", client_id, encoded_scope);
    let resp = client
        .post("https://open.littleskin.cn/oauth/device_code")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(OAuthError::LittleSkin(err_text));
    }
    let code_resp: DeviceCodeResponse = resp.json().await?;
    Ok(code_resp)
}

/// Helper: Use default client and CLIENT_ID for device code flow
pub async fn request_device_code_default(scope: &str) -> Result<DeviceCodeResponse, OAuthError> {
    let client = Client::new();
    request_device_code(&client, CLIENT_ID, scope).await
}

/// Helper: Poll for device token, then fetch user info and build Account
pub async fn poll_device_token_default(
    device_code: String,
    expires_in: u64,
) -> Result<crate::auth::littleskin::Account, OAuthError> {
    let client = Client::new();
    let token_resp = poll_device_token(&client, CLIENT_ID, &device_code, 5, expires_in).await?;
    let access_token = token_resp
        .access_token
        .ok_or_else(|| OAuthError::LittleSkin("No access_token in response".to_string()))?;
    let user_info = get_user_info(&client, &access_token).await?;
    Ok(crate::auth::littleskin::Account::Account(crate::auth::littleskin::AccountData {
        access_token: Some(access_token.clone()),
        uuid: user_info.id.to_string(),
        username: user_info.username.clone(),
        nice_username: user_info.username,
        refresh_token: token_resp.refresh_token.unwrap_or_default(),
        needs_refresh: false,
        account_type: crate::auth::AccountType::LittleSkin,
    }))
}


/// Step 2: Poll for token
pub async fn poll_device_token(
    client: &Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<DeviceTokenResponse, OAuthError> {
    use tokio::time::{sleep, Duration, Instant};
    let start = Instant::now();
    loop {
        if start.elapsed().as_secs() > expires_in {
            return Err(OAuthError::LittleSkin("Device code expired".to_string()));
        }
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
            ("client_id", client_id),
        ];
        let resp = client
            .post("https://open.littleskin.cn/oauth/token")
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await?;
        let token_resp: DeviceTokenResponse = resp.json().await?;
        if let Some(ref err) = token_resp.error {
            match err.as_str() {
                "authorization_pending" => {
                    sleep(Duration::from_secs(interval)).await;
                    continue;
                }
                "slow_down" => {
                    sleep(Duration::from_secs(interval + 2)).await;
                    continue;
                }
                "expired_token" | "access_denied" => {
                    return Err(OAuthError::LittleSkin(token_resp.error_description.unwrap_or(err.clone())));
                }
                _ => {
                    return Err(OAuthError::LittleSkin(token_resp.error_description.unwrap_or(err.clone())));
                }
            }
        }
        if let Some(access_token) = &token_resp.access_token {
            return Ok(token_resp);
        }
        // If no error and no token, wait and retry
        sleep(Duration::from_secs(interval)).await;
    }
}