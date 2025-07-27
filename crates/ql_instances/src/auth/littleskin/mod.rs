use crate::auth::alt::AccountResponse;

use super::AccountData;
use ql_core::{err, info, pt, IntoJsonError, IntoStringError, RequestError, CLIENT};

pub use super::alt::{Account, AccountResponseError, Error};
use serde::Serialize;
pub mod oauth;

const CLIENT_ID: &str = "1160";

#[derive(Serialize)]
struct Agent<'a> {
    name: &'a str,
    version: u8,
}

fn strip_littleskin_suffix(username: &str) -> &str {
    username.strip_suffix(" (littleskin)").unwrap_or(username)
}

pub async fn login_new(email: String, password: String) -> Result<Account, Error> {
    // NOTE: It says email, but both username and email are accepted
    info!("Logging into littleskin... ({email})");
    let response = CLIENT
        .post("https://littleskin.cn/api/yggdrasil/authserver/authenticate")
        .json(&serde_json::json!({
            "agent" : Agent{
                name: "Minecraft",
                version: 1
            },
            "username": &email,
            "password": &password,
            "clientToken": CLIENT_ID
        }))
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

    let account_response = match serde_json::from_str::<AccountResponse>(&text).json(text.clone()) {
        Ok(n) => n,
        Err(err) => {
            if let Ok(res_err) = serde_json::from_str::<AccountResponseError>(&text).json(text) {
                if res_err.error == "ForbiddenOperationException"
                    && res_err.errorMessage == "Account protected with two factor auth."
                {
                    return Ok(Account::NeedsOTP);
                }
            }
            return Err(err.into());
        }
    };

    let entry = get_keyring_entry(strip_littleskin_suffix(&email))?;
    entry.set_password(&account_response.accessToken)?;

    Ok(Account::Account(AccountData {
        access_token: Some(account_response.accessToken.clone()),
        uuid: account_response.selectedProfile.id,

        username: email,
        nice_username: account_response.selectedProfile.name,

        refresh_token: account_response.accessToken,
        needs_refresh: false,
        account_type: super::AccountType::LittleSkin,
    }))
}

pub fn read_refresh_token(username: &str) -> Result<String, Error> {
    let entry = get_keyring_entry(strip_littleskin_suffix(username))?;
    Ok(entry.get_password()?)
}

pub async fn login_refresh(email: String, refresh_token: String) -> Result<AccountData, Error> {
    // NOTE: It says email, but both username and email are accepted

    pt!("Refreshing littleskin.cn account...");
    let entry = get_keyring_entry(strip_littleskin_suffix(&email))?;

    let response = CLIENT
        .post("https://littleskin.cn/api/yggdrasil/authserver/refresh")
        .json(&serde_json::json!({
            "agent" : Agent{
                name: "Minecraft",
                version: 1
            },
            "accessToken": refresh_token,
            "clientToken": CLIENT_ID
        }))
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

    let account_response = serde_json::from_str::<AccountResponse>(&text).json(text.clone())?;
    entry.set_password(&account_response.accessToken)?;

    Ok(AccountData {
        access_token: Some(account_response.accessToken.clone()),
        uuid: account_response.selectedProfile.id,

        username: email,
        nice_username: account_response.selectedProfile.name,

        refresh_token: account_response.accessToken,
        needs_refresh: false,
        account_type: super::AccountType::LittleSkin,
    })
}

fn get_keyring_entry(username: &str) -> Result<keyring::Entry, Error> {
    Ok(keyring::Entry::new(
        "QuantumLauncher",
        &format!("{username}#littleskin"),
    )?)
}

pub fn logout(username: &str) -> Result<(), String> {
    let entry = get_keyring_entry(strip_littleskin_suffix(username)).strerr()?;
    if let Err(err) = entry.delete_credential() {
        err!("Couldn't remove littleskin account credential (Username: {username}):\n{err}");
    }
    Ok(())
}
