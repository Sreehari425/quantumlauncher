use ql_core::{err, IntoStringError};
use std::fmt::Display;

mod alt;
pub mod authlib;
pub mod encrypted_store;
pub mod ms;
pub mod token_store;
pub mod yggdrasil;
pub use authlib::get_authlib_injector;
pub use token_store::TokenStorageMethod;

#[derive(Debug, Clone)]
pub struct AccountData {
    pub access_token: Option<String>,
    pub uuid: String,
    pub refresh_token: String,
    pub needs_refresh: bool,

    pub username: String,
    pub nice_username: String,

    pub account_type: AccountType,
}

impl AccountData {
    #[must_use]
    pub fn get_username_modified(&self) -> String {
        let suffix = match self.account_type {
            AccountType::Microsoft => "",
            AccountType::ElyBy => " (elyby)",
            AccountType::LittleSkin => " (littleskin)",
        };
        format!("{}{suffix}", self.nice_username)
    }

    #[must_use]
    pub fn get_authlib_url(&self) -> Option<&'static str> {
        match self.account_type {
            AccountType::Microsoft => None,
            AccountType::ElyBy => Some("ely.by"),
            AccountType::LittleSkin => Some("https://littleskin.cn/api/yggdrasil"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AccountType {
    Microsoft,
    ElyBy,
    LittleSkin,
}

impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AccountType::Microsoft => "Microsoft",
                AccountType::ElyBy => "ElyBy",
                AccountType::LittleSkin => "LittleSkin",
            }
        )
    }
}

impl AccountType {
    #[must_use]
    pub fn yggdrasil_authenticate(self) -> &'static str {
        match self {
            AccountType::Microsoft => unreachable!(),
            AccountType::ElyBy => "https://authserver.ely.by/auth/authenticate",
            AccountType::LittleSkin => {
                "https://littleskin.cn/api/yggdrasil/authserver/authenticate"
            }
        }
    }

    #[must_use]
    pub fn yggdrasil_refresh(self) -> &'static str {
        match self {
            AccountType::Microsoft => unreachable!(),
            AccountType::ElyBy => "https://authserver.ely.by/auth/refresh",
            AccountType::LittleSkin => "https://littleskin.cn/api/yggdrasil/authserver/refresh",
        }
    }

    #[must_use]
    pub fn yggdrasil_needs_agent_field(self) -> bool {
        match self {
            AccountType::Microsoft | AccountType::ElyBy => false,
            AccountType::LittleSkin => true,
        }
    }

    fn get_keyring_entry(self, username: &str) -> Result<keyring::Entry, KeyringError> {
        Ok(keyring::Entry::new(
            "QuantumLauncher",
            &self.create_storage_key(username),
        )?)
    }

    #[must_use]
    pub fn create_storage_key(self, username: &str) -> String {
        format!(
            "{username}{}",
            match self {
                AccountType::Microsoft => "",
                AccountType::ElyBy => "#elyby",
                AccountType::LittleSkin => "#littleskin",
            }
        )
    }

    #[must_use]
    pub(crate) fn get_client_id(self) -> &'static str {
        match self {
            AccountType::Microsoft => ms::CLIENT_ID,
            AccountType::ElyBy => "quantumlauncher1",
            AccountType::LittleSkin => "1160",
        }
    }

    #[must_use]
    pub fn strip_name(self, name: &str) -> &str {
        match self {
            AccountType::Microsoft => name,
            AccountType::ElyBy => name.strip_suffix(" (elyby)").unwrap_or(name),
            AccountType::LittleSkin => name.strip_suffix(" (littleskin)").unwrap_or(name),
        }
    }
}

impl AccountData {
    #[must_use]
    pub fn is_elyby(&self) -> bool {
        matches!(self.account_type, AccountType::ElyBy)
    }
    #[must_use]
    pub fn is_littleskin(&self) -> bool {
        matches!(self.account_type, AccountType::LittleSkin)
    }
    #[must_use]
    pub fn is_microsoft(&self) -> bool {
        matches!(self.account_type, AccountType::Microsoft)
    }
}

#[derive(Debug, thiserror::Error)]
pub struct KeyringError(#[from] pub keyring::Error);

impl Display for KeyringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Account keyring error:")?;
        match &self.0 {
            #[cfg(target_os = "linux")]
            keyring::Error::PlatformFailure(error)
                if error.to_string().contains("The name is not activatable") =>
            {
                write!(f, "{error}\n\nTry installing gnome-keyring and libsecret packages\n(may be called differently depending on your distro)")
            }
            #[cfg(target_os = "linux")]
            keyring::Error::NoStorageAccess(error)
                if error.to_string().contains("no result found") =>
            {
                write!(
                    f,
                    r#"{error}

Install the "seahorse" app and open it,
Check for "Login" in the sidebar.
If it's there, make sure it's unlocked (right-click -> Unlock)

If it's not there, click on + then "Password Keyring",
and name it "Login" and put your preferred password

Now after this, in the sidebar, right click it and click "Set as Default""#
                )
            }

            _ => write!(f, "{}", self.0),
        }
    }
}

/// Read a refresh token from the keyring directly (bypass token_store routing).
pub fn read_refresh_token_keyring(
    username: &str,
    account_type: AccountType,
) -> Result<String, KeyringError> {
    let entry = account_type.get_keyring_entry(username)?;
    let refresh_token = entry.get_password()?;
    Ok(refresh_token)
}

/// Read a refresh token using the currently active backend.
pub fn read_refresh_token(
    username: &str,
    account_type: AccountType,
) -> Result<String, token_store::TokenStoreError> {
    token_store::read_token(username, account_type)
}

/// Read a refresh token from a specific backend.
pub fn read_refresh_token_from(
    username: &str,
    account_type: AccountType,
    method: TokenStorageMethod,
) -> Result<String, token_store::TokenStoreError> {
    token_store::read_token_from(username, account_type, method)
}

/// Delete a credential from the keyring directly.
pub fn logout_keyring(username: &str, account_type: AccountType) -> Result<(), String> {
    let entry = account_type.get_keyring_entry(username).strerr()?;
    if let Err(err) = entry.delete_credential() {
        err!("Couldn't remove {account_type} account credential (Username: {username}):\n{err}");
    }
    Ok(())
}

/// Delete a credential using the currently active backend.
pub fn logout(username: &str, account_type: AccountType) -> Result<(), String> {
    token_store::delete_token(username, account_type).map_err(|e| e.to_string())
}

// /// Delete a credential from a specific backend.
// fn logout_from(
//     username: &str,
//     account_type: AccountType,
//     method: TokenStorageMethod,
// ) -> Result<(), String> {
//     token_store::delete_token_from(username, account_type, method).map_err(|e| e.to_string())
// }
