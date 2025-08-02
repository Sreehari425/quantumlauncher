use crate::auth::AccountData;

use super::alt::{Error};
pub use super::alt::Account;
use ql_core::{err, info, pt, IntoJsonError, IntoStringError, RequestError, CLIENT};
use serde::{Deserialize, Serialize};

const CLIENT_ID: &str = "quantumlauncher1";

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize, Clone, Debug)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Clone, Debug)]
struct UserInfo {
    uid: i32,
    email: String,
    nickname: String,
    avatar: i32,
    score: i32,
    permission: i32,
    #[serde(rename = "last_sign_at")]
    last_sign_at: String,
    #[serde(rename = "register_at")]
    register_at: String,
    verified: bool,
}

fn strip_blessing_skin_suffix(username: &str) -> &str {
    username.strip_suffix(" (blessing)").unwrap_or(username)
}

pub async fn login_new(email: String, password: String, base_url: String) -> Result<Account, Error> {
    info!("Logging into blessing skin server... ({email}) at {base_url}");
    
    // Construct the login URL from the base URL
    let login_url = format!("{}/api/auth/login", base_url.trim_end_matches('/'));
    
    let response = CLIENT
        .post(&login_url)
        .json(&LoginRequest {
            email: email.clone(),
            password,
        })
        .send()
        .await?;

    let text = if response.status().is_success() {
        response.text().await?
    } else {
        return Err(RequestError::DownloadError {
            code: response.status(),
            url: response.url().clone(),
        }
        .into());
    };

    let login_response = serde_json::from_str::<LoginResponse>(&text).json(text)?;
    
    // Get user information
    let user_info_url = format!("{}/api/user", base_url.trim_end_matches('/'));
    let user_response = CLIENT
        .get(&user_info_url)
        .header("Authorization", format!("Bearer {}", login_response.token))
        .send()
        .await?;

    let user_text = if user_response.status().is_success() {
        user_response.text().await?
    } else {
        return Err(RequestError::DownloadError {
            code: user_response.status(),
            url: user_response.url().clone(),
        }
        .into());
    };

    let user_info = serde_json::from_str::<UserInfo>(&user_text).json(user_text)?;

    let entry = get_keyring_entry(strip_blessing_skin_suffix(&email), &base_url)?;
    entry.set_password(&login_response.token)?;

    Ok(Account::Account(AccountData {
        access_token: Some(login_response.token.clone()),
        uuid: user_info.uid.to_string(), // Use UID as UUID since Blessing Skin doesn't use Minecraft UUIDs
        username: email,
        nice_username: user_info.nickname,
        refresh_token: login_response.token,
        needs_refresh: false,
        account_type: super::AccountType::BlessingSkin,
        custom_auth_url: Some(base_url),
    }))
}
pub fn read_refresh_token(username: &str, base_url: &str) -> Result<String, Error> {
    let entry = get_keyring_entry(strip_blessing_skin_suffix(username), base_url)?;
    Ok(entry.get_password()?)
}

pub async fn login_refresh(email: String, refresh_token: String, base_url: String) -> Result<AccountData, Error> {
    pt!("Refreshing blessing skin account...");
    let entry = get_keyring_entry(strip_blessing_skin_suffix(&email), &base_url)?;

    // Construct the refresh URL from the base URL
    let refresh_url = format!("{}/api/auth/refresh", base_url.trim_end_matches('/'));

    let response = CLIENT
        .post(&refresh_url)
        .header("Authorization", format!("Bearer {}", refresh_token))
        .send()
        .await?;

    let text = if response.status().is_success() {
        response.text().await?
    } else {
        return Err(RequestError::DownloadError {
            code: response.status(),
            url: response.url().clone(),
        }
        .into());
    };

    let login_response = serde_json::from_str::<LoginResponse>(&text).json(text)?;
    
    // Get updated user information
    let user_info_url = format!("{}/api/user", base_url.trim_end_matches('/'));
    let user_response = CLIENT
        .get(&user_info_url)
        .header("Authorization", format!("Bearer {}", login_response.token))
        .send()
        .await?;

    let user_text = if user_response.status().is_success() {
        user_response.text().await?
    } else {
        return Err(RequestError::DownloadError {
            code: user_response.status(),
            url: user_response.url().clone(),
        }
        .into());
    };

    let user_info = serde_json::from_str::<UserInfo>(&user_text).json(user_text)?;
    
    entry.set_password(&login_response.token)?;

    Ok(AccountData {
        access_token: Some(login_response.token.clone()),
        uuid: user_info.uid.to_string(),
        username: email,
        nice_username: user_info.nickname,
        refresh_token: login_response.token,
        needs_refresh: false,
        account_type: super::AccountType::BlessingSkin,
        custom_auth_url: Some(base_url),
    })
}

fn get_keyring_entry(username: &str, base_url: &str) -> Result<keyring::Entry, Error> {
    // Use a hash of the base_url to make it unique per server
    let url_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        base_url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };
    
    Ok(keyring::Entry::new(
        "QuantumLauncher",
        &format!("{username}#blessing-{url_hash}"),
    )?)
}

pub async fn logout(username: &str, base_url: &str) -> Result<(), String> {
    let username_stripped = strip_blessing_skin_suffix(username);
    let entry = get_keyring_entry(username_stripped, base_url).strerr()?;
    
    // Try to get the JWT token for proper logout
    if let Ok(token) = entry.get_password() {
        // Construct the logout URL from the base URL
        let logout_url = format!("{}/api/auth/logout", base_url.trim_end_matches('/'));
        
        // Send logout request to the server
        let _ = CLIENT
            .post(&logout_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await; // We don't care if this fails, we'll remove the local token anyway
    }
    
    // Remove the credential from keyring
    if let Err(err) = entry.delete_credential() {
        err!("Couldn't remove blessing skin account credential (Username: {username_stripped}):\n{err}");
    }
    
    Ok(())
}
