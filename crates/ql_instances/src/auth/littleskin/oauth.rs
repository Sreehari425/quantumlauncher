use keyring;
use ql_core::IntoJsonError;
use ql_reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{Error, CLIENT_ID};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenResponse {
    pub token_type: String,
    pub expires_in: u64,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    #[serde(rename = "uid")]
    pub id: u64,
    #[serde(rename = "nickname")]
    pub username: String,
    pub email: Option<String>,
}

/// Step 1: Generate the authorization URL for the user to visit
pub fn authorization_url(scope: &str) -> String {
    format!(
        "https://littleskin.cn/oauth/authorize?client_id={CLIENT_ID}&response_type=code&scope={scope}"
    )
}

/// Step 2: Exchange the authorization code for tokens
pub async fn exchange_code_for_token(client: &Client, code: &str) -> Result<TokenResponse, Error> {
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
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
    }
    let token = resp.text().await?;
    let token: TokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}

/// Step 3: Refresh the access token using the refresh token
pub async fn refresh_token(client: &Client, refresh_token: &str) -> Result<TokenResponse, Error> {
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
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
    }
    let token = resp.text().await?;
    let token: TokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}

/// Step 4: Get user info using the access token
pub async fn get_user_info(client: &Client, access_token: &str) -> Result<UserInfo, Error> {
    let resp = client
        .get("https://littleskin.cn/api/user")
        .header("Accept", "application/json")
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
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
) -> Result<DeviceCodeResponse, Error> {
    // Build body as LittleSkin expects: "client_id={client_id}&\nscope={scope}"
    let encoded_scope = urlencoding::encode(scope);
    let body = format!("client_id={client_id}&scope={encoded_scope}");
    let resp = client
        .post("https://open.littleskin.cn/oauth/device_code")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
    }

    let code_resp = resp.text().await?;
    let code_resp: DeviceCodeResponse = serde_json::from_str(&code_resp).json(code_resp)?;
    Ok(code_resp)
}

/// Helper: Use default client and CLIENT_ID for device code flow
pub async fn request_device_code_default(scope: &str) -> Result<DeviceCodeResponse, Error> {
    let client = Client::new();
    request_device_code(&client, CLIENT_ID, scope).await
}

/// Helper: Poll for device token, then fetch user info and build Account
/// Helper: Poll for device token, then fetch user info, create Minecraft token and build Account
pub async fn poll_device_token_default(
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> Result<crate::auth::littleskin::Account, Error> {
    let client = Client::new();
    // Step A: exchange device_code for OAuth access_token
    let token_resp =
        poll_device_token(&client, CLIENT_ID, &device_code, interval, expires_in).await?;
    let oauth_access_token = token_resp
        .access_token
        .ok_or_else(|| Error::LittleSkin("No access_token in response".to_string()))?;
    // Step B: fetch user info (we need uuid & username)
    let user_info = get_user_info(&client, &oauth_access_token).await?;
    // Step C: exchange OAuth token for a Yggdrasil/Minecraft token (needed for actual game login)
    // Sub step get UUID
    let profile = get_minecraft_profile(&client, &oauth_access_token).await?;
    let uuid = profile.id;
    let mut mc_token_resp = create_minecraft_token(&client, &oauth_access_token, &uuid).await?;
    // If server didn't include selectedProfile, fetch via sessionserver
    if mc_token_resp.selected_profile.is_none() {
        if let Ok(profile) = get_minecraft_profile(&client, &oauth_access_token).await {
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

async fn get_minecraft_profile(
    client: &Client,
    oauth_access_token: &str,
) -> Result<MinecraftProfile, Error> {
    let resp = client
        .get("https://littleskin.cn/api/yggdrasil/sessionserver/session/minecraft/profile")
        .header("Accept", "application/json")
        .bearer_auth(oauth_access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
    }

    // API returns an array of profiles;
    let list = resp.text().await?;
    let mut list: Vec<MinecraftProfile> = serde_json::from_str(&list).json(list)?;
    list.pop()
        .ok_or_else(|| Error::LittleSkin("No Minecraft profile returned".to_string()))
}

async fn create_minecraft_token(
    client: &Client,
    oauth_access_token: &str,
    uuid: &str,
) -> Result<MinecraftTokenResponse, Error> {
    let resp = client
        .post("https://littleskin.cn/api/yggdrasil/authserver/oauth")
        .bearer_auth(oauth_access_token)
        .header("Accept", "application/json")
        .json(&serde_json::json!({ "uuid": uuid.to_string() }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::LittleSkin(err_text));
    }
    let token = resp.text().await?;
    let token: MinecraftTokenResponse = serde_json::from_str(&token).json(token)?;
    Ok(token)
}

/// Step 2: Poll for token
pub async fn poll_device_token(
    client: &Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<DeviceTokenResponse, Error> {
    use tokio::time::{sleep, Duration, Instant};
    let start = Instant::now();
    loop {
        if start.elapsed().as_secs() > expires_in {
            return Err(Error::LittleSkin("Device code expired".to_string()));
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
        let status = resp.status();
        if status.as_u16() >= 500 {
            sleep(Duration::from_secs(interval)).await;
            continue;
        }
        let text_body = resp.text().await?;
        // LittleSkin sometimes returns error JSON with 4xx codes. We'll still try to parse.
        let token_resp: DeviceTokenResponse = match serde_json::from_str(&text_body) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::LittleSkin(format!(
                    "Unexpected response from LittleSkin: {text_body}"
                )));
            }
        };
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
