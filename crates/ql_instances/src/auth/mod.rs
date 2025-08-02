use std::fmt::Display;

use crate::auth;
mod alt;
pub mod authlib;
pub mod blessing_skin;
pub mod elyby;
pub mod littleskin;
pub mod ms;
pub use authlib::get_authlib_injector;

#[derive(Debug, Clone)]
pub struct AccountData {
    pub access_token: Option<String>,
    pub uuid: String,
    pub refresh_token: String,
    pub needs_refresh: bool,

    pub username: String,
    pub nice_username: String,

    pub account_type: AccountType,
    pub custom_auth_url: Option<String>, // For blessing skin servers
}

impl AccountData {
    pub fn get_username_modified(&self) -> String {
        let suffix = match self.account_type {
            auth::AccountType::Microsoft => "",
            auth::AccountType::ElyBy => " (elyby)",
            auth::AccountType::LittleSkin => " (littleskin)",
            auth::AccountType::BlessingSkin => " (blessing)",
        };
        format!("{}{suffix}", self.username)
    }

    pub fn get_authlib_url(&self) -> Option<&'static str> {
        match self.account_type {
            AccountType::Microsoft => None,
            AccountType::ElyBy => Some("ely.by"),
            AccountType::LittleSkin => Some("https://littleskin.cn/api/yggdrasil"),
            AccountType::BlessingSkin => None, // Custom URL will be handled separately
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountType {
    Microsoft,
    ElyBy,
    LittleSkin,
    BlessingSkin,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AccountType::Microsoft => "Microsoft",
                AccountType::ElyBy => "ElyBy",
                AccountType::LittleSkin => "LittleSkin",
                AccountType::BlessingSkin => "BlessingSkin",
            }
        )
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
    #[must_use]
    pub fn is_blessing_skin(&self) -> bool {
        matches!(self.account_type, AccountType::BlessingSkin)
    }
}

#[derive(Debug, thiserror::Error)]
pub struct KeyringError(pub keyring::Error);

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
