use ql_reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CLIENT_ID: &str = "YOUR_CLIENT_ID";
pub const CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";
pub const REDIRECT_URI: &str = "YOUR_REDIRECT_URI";

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
pub fn authorization_url(scope: &str) -> String {
    format!(
        "https://littleskin.cn/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
        CLIENT_ID, REDIRECT_URI, scope
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
        ("client_secret", CLIENT_SECRET),
        ("redirect_uri", REDIRECT_URI),
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
        ("client_secret", CLIENT_SECRET),
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

// Example usage (async context):
// let client = reqwest::Client::new();
// let url = authorization_url("User.Read");
// println!("Visit this URL to authorize: {}", url);
// // After user authorizes and you get the code:
// let token = exchange_code_for_token(&client, "CODE").await?;
// let refreshed = refresh_token(&client, &token.refresh_token).await?;
// let user = get_user_info(&client, &token.access_token).await?; 