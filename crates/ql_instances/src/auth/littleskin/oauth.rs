use keyring;
use ql_core::{IntoJsonError, RequestError, CLIENT};
use serde::{Deserialize, Serialize};

use crate::auth::alt::OauthError;

use super::{Error, CLIENT_ID};

pub const SCOPE: &str =
    "Yggdrasil.PlayerProfiles.Read Yggdrasil.Server.Join Yggdrasil.MinecraftToken.Create User.Read";

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TokenResponse {
    token_type: String,
    expires_in: u64,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserInfo {
    #[serde(rename = "uid")]
    id: u64,
    #[serde(rename = "nickname")]
    username: String,
    email: Option<String>,
}

/* /// Step 1: Generate the authorization URL for the user to visit
fn authorization_url(scope: &str) -> String {
    format!(
        "https://littleskin.cn/oauth/authorize?client_id={CLIENT_ID}&response_type=code&scope={scope}"
    )
}
/// Step 2: Exchange the authorization code for tokens
async fn exchange_code_for_token(client: &Client, code: &str) -> Result<TokenResponse, Error> {
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("code", code),
    ];
    let resp = client
        .post("https://littleskin.cn/oauth/token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }
    let token = resp.text().await?;
    let token: TokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}
/// Step 3: Refresh the access token using the refresh token
async fn refresh_token(client: &Client, refresh_token: &str) -> Result<TokenResponse, Error> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
    ];
    let resp = client
        .post("https://littleskin.cn/oauth/token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }
    let token = resp.text().await?;
    let token: TokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}*/

/// Step 4: Get user info using the access token
async fn get_user_info(access_token: &str) -> Result<UserInfo, Error> {
    let resp = CLIENT
        .get("https://littleskin.cn/api/user")
        .header("Accept", "application/json")
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }
    let user = resp.text().await?;
    let user: UserInfo = serde_json::from_str(&user).json(user)?;
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
struct DeviceTokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Step 1: Request device code
pub async fn request_device_code() -> Result<DeviceCodeResponse, Error> {
    let encoded_scope = urlencoding::encode(SCOPE);
    let body = format!("client_id={CLIENT_ID}&scope={encoded_scope}");
    let resp = CLIENT
        .post("https://open.littleskin.cn/oauth/device_code")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }

    let code_resp = resp.text().await?;
    let code_resp: DeviceCodeResponse = serde_json::from_str(&code_resp).json(code_resp)?;
    Ok(code_resp)
}

/// Helper: Poll for device token, then fetch user info, create Minecraft token and build Account
pub async fn poll_device_token(
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> Result<crate::auth::littleskin::Account, Error> {
    // Step A: exchange device_code for OAuth access_token
    let token_resp = get_device_token(&device_code, interval, expires_in).await?;
    let oauth_access_token = token_resp.access_token.ok_or(OauthError::NoAccessToken)?;

    // Step B: fetch user info (we need uuid & username)
    let user_info = get_user_info(&oauth_access_token).await?;

    // Step C: exchange OAuth token for a Yggdrasil/Minecraft token (needed for actual game login)
    // Sub step get UUID
    let profile = get_minecraft_profile(&oauth_access_token).await?;
    let uuid = profile.id;
    let mut mc_token_resp = create_minecraft_token(&oauth_access_token, &uuid).await?;
    // If server didn't include selectedProfile, fetch via sessionserver
    if mc_token_resp.selected_profile.is_none() {
        if let Ok(profile) = get_minecraft_profile(&oauth_access_token).await {
            mc_token_resp.selected_profile = Some(profile);
        }
    }

    // Store Minecraft token in keyring (same convention as password flow)
    keyring::Entry::new(
        "QuantumLauncher",
        &format!("{}#littleskin", user_info.username),
    )
    .and_then(|e| e.set_password(&mc_token_resp.access_token))?;

    // Build account data compatible with existing flows
    Ok(crate::auth::littleskin::Account::Account(
        crate::auth::littleskin::AccountData {
            access_token: Some(mc_token_resp.access_token.clone()),
            uuid: mc_token_resp
                .selected_profile
                .as_ref()
                .map(|p| p.id.clone())
                .unwrap_or_default(),
            username: user_info.username.clone(),
            nice_username: mc_token_resp
                .selected_profile
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| user_info.username.clone()),
            refresh_token: mc_token_resp.access_token,
            needs_refresh: false,
            account_type: crate::auth::AccountType::LittleSkin,
            custom_auth_url: None,
        },
    ))
}

#[derive(Debug, Deserialize)]
struct MinecraftTokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "selectedProfile")]
    selected_profile: Option<MinecraftProfile>,
}

#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    #[serde(default)]
    _properties: Option<Vec<serde_json::Value>>, // ignored
}

async fn get_minecraft_profile(oauth_access_token: &str) -> Result<MinecraftProfile, Error> {
    let resp = CLIENT
        .get("https://littleskin.cn/api/yggdrasil/sessionserver/session/minecraft/profile")
        .header("Accept", "application/json")
        .bearer_auth(oauth_access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }

    // API returns an array of profiles;
    let list = resp.text().await?;
    let mut list: Vec<MinecraftProfile> = serde_json::from_str(&list).json(list)?;
    list.pop().ok_or(OauthError::NoMinecraftProfile.into())
}

async fn create_minecraft_token(
    oauth_access_token: &str,
    uuid: &str,
) -> Result<MinecraftTokenResponse, Error> {
    let resp = CLIENT
        .post("https://littleskin.cn/api/yggdrasil/authserver/oauth")
        .bearer_auth(oauth_access_token)
        .header("Accept", "application/json")
        .json(&serde_json::json!({ "uuid": uuid.to_string() }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(RequestError::DownloadError {
            code: resp.status(),
            url: resp.url().clone(),
        }
        .into());
    }
    let token = resp.text().await?;
    let token: MinecraftTokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}

async fn get_device_token(
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<DeviceTokenResponse, Error> {
    use tokio::time::{sleep, Duration, Instant};
    let start = Instant::now();
    loop {
        if start.elapsed().as_secs() > expires_in {
            return Err(OauthError::DeviceCodeExpired.into());
        }
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
            ("client_id", CLIENT_ID),
        ];
        let resp = CLIENT
            .post("https://open.littleskin.cn/oauth/token")
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await?;
        let status = resp.status();
        if status.as_u16() >= 500 {
            sleep(Duration::from_secs(interval)).await;
            continue;
        }
        let text_body = resp.text().await?;

        // LittleSkin sometimes returns error JSON with 4xx codes. We'll still try to parse.
        let token_resp: DeviceTokenResponse = serde_json::from_str(&text_body)
            .map_err(|_| OauthError::UnexpectedResponse(text_body))?;

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
                    return Err(Error::LittleSkin(
                        token_resp.error_description.unwrap_or(err.clone()),
                    ));
                }
                _ => {
                    return Err(Error::LittleSkin(
                        token_resp.error_description.unwrap_or(err.clone()),
                    ));
                }
            }
        }
        if let Some(_access_token) = &token_resp.access_token {
            return Ok(token_resp);
        }
        // If no error and no token, wait and retry
        sleep(Duration::from_secs(interval)).await;
    }
}
