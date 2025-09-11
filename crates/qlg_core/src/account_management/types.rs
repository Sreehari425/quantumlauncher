//! Core types for account management

use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents different account providers supported by the launcher
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountProvider {
    /// Microsoft accounts (premium accounts)
    Microsoft,
    /// ElyBy alternative authentication server
    ElyBy,
    /// LittleSkin alternative authentication server
    LittleSkin,
}

impl Display for AccountProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountProvider::Microsoft => write!(f, "Microsoft"),
            AccountProvider::ElyBy => write!(f, "ElyBy"),
            AccountProvider::LittleSkin => write!(f, "LittleSkin"),
        }
    }
}

impl AccountProvider {
    /// Get the display name with branding
    pub fn display_name(&self) -> &'static str {
        match self {
            AccountProvider::Microsoft => "Microsoft",
            AccountProvider::ElyBy => "ElyBy",
            AccountProvider::LittleSkin => "LittleSkin",
        }
    }

    /// Check if this provider requires premium Minecraft ownership
    pub fn requires_ownership(&self) -> bool {
        matches!(self, AccountProvider::Microsoft)
    }

    /// Check if this provider supports OAuth flow
    pub fn supports_oauth(&self) -> bool {
        matches!(
            self,
            AccountProvider::Microsoft | AccountProvider::LittleSkin
        )
    }

    /// Check if this provider supports username/password authentication
    pub fn supports_credentials(&self) -> bool {
        matches!(self, AccountProvider::ElyBy | AccountProvider::LittleSkin)
    }
}

/// Account information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier for the account
    pub uuid: String,
    /// Username used for login
    pub username: String,
    /// Display name (may differ from username)
    pub display_name: String,
    /// Account provider
    pub provider: AccountProvider,
    /// Current access token (if available)
    pub access_token: Option<String>,
    /// Whether the token needs refreshing
    pub needs_refresh: bool,
}

impl Account {
    /// Get a modified username for display purposes
    pub fn display_username(&self) -> String {
        let suffix = match self.provider {
            AccountProvider::Microsoft => "",
            AccountProvider::ElyBy => " (ElyBy)",
            AccountProvider::LittleSkin => " (LittleSkin)",
        };
        format!("{}{}", self.display_name, suffix)
    }

    /// Get the authlib URL for this account (if applicable)
    pub fn authlib_url(&self) -> Option<&'static str> {
        match self.provider {
            AccountProvider::Microsoft => None,
            AccountProvider::ElyBy => Some("ely.by"),
            AccountProvider::LittleSkin => Some("https://littleskin.cn/api/yggdrasil"),
        }
    }
}

/// Authentication result from login attempts
#[derive(Debug, Clone)]
pub enum AuthResult {
    /// Successful authentication with account data
    Success(Account),
    /// Two-factor authentication required
    RequiresTwoFactor,
    /// Authentication failed with error message
    Failed(String),
}

/// Login credentials for username/password authentication
#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

/// OAuth authentication flow data
#[derive(Debug, Clone)]
pub struct OAuthFlow {
    pub verification_uri: String,
    pub user_code: String,
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Errors that can occur during account management
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Account doesn't own Minecraft")]
    NoMinecraftOwnership,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Provider not supported for this operation")]
    UnsupportedProvider,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Operation cancelled by user")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, AccountError>;
