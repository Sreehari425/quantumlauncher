//! Core types for account management

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure wrapper for sensitive strings that automatically zeroizes on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: Vec<u8>,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            inner: value.into().into_bytes(),
        }
    }

    /// Get the string value (use sparingly and clear result when done)
    pub fn expose_secret(&self) -> String {
        // This is intentionally unsafe to use - consumers should be careful
        String::from_utf8_lossy(&self.inner).to_string()
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Append another string (for TOTP concatenation)
    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Push a single character
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.inner.extend_from_slice(s.as_bytes());
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureString")
            .field("len", &self.len())
            .finish_non_exhaustive()
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl Serialize for SecureString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the exposed secret - this is intentionally dangerous
        // and should only be used when absolutely necessary
        serializer.serialize_str(&self.expose_secret())
    }
}

impl<'de> Deserialize<'de> for SecureString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SecureString::new(s))
    }
}

/// NewType wrapper for usernames to ensure type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Create a new username with validation
    pub fn new(username: impl Into<String>) -> std::result::Result<Self, String> {
        let username = username.into();

        // Basic validation
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if username.len() < 3 || username.len() > 50 {
            return Err("Username must be between 3 and 50 characters".to_string());
        }

        Ok(Self(username))
    }

    /// Create a username without validation (for trusted sources)
    pub fn new_unchecked(username: impl Into<String>) -> Self {
        Self(username.into())
    }

    /// Get the username as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Username> for String {
    fn from(username: Username) -> String {
        username.0
    }
}

impl From<&str> for Username {
    fn from(s: &str) -> Self {
        Self::new_unchecked(s)
    }
}

impl From<String> for Username {
    fn from(s: String) -> Self {
        Self::new_unchecked(s)
    }
}

/// NewType wrapper for access tokens to ensure type safety
/// Automatically zeroizes on drop to prevent token leaks
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct AccessToken(SecureString);

impl AccessToken {
    /// Create a new access token
    pub fn new(token: impl Into<String>) -> Self {
        Self(SecureString::new(token.into()))
    }

    /// Get the token as a string slice (use carefully and clear result)
    pub fn as_str(&self) -> String {
        self.0.expose_secret()
    }

    /// Check if the token is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the first few characters for logging (security-safe)
    pub fn preview(&self) -> String {
        let token = self.0.expose_secret();
        if token.len() > 8 {
            format!("{}...", &token[..8])
        } else {
            "***".to_string()
        }
    }
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessToken")
            .field("preview", &self.preview())
            .finish()
    }
}

impl From<AccessToken> for String {
    fn from(token: AccessToken) -> String {
        token.0.expose_secret()
    }
}

impl From<String> for AccessToken {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for AccessToken {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// NewType wrapper for UUIDs to ensure type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountUuid(String);

impl AccountUuid {
    /// Create a new UUID
    pub fn new(uuid: impl Into<String>) -> Self {
        Self(uuid.into())
    }

    /// Get the UUID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create the standard offline UUID
    pub fn offline() -> Self {
        Self::new("00000000-0000-0000-0000-000000000000")
    }
}

impl Display for AccountUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<AccountUuid> for String {
    fn from(uuid: AccountUuid) -> String {
        uuid.0
    }
}

impl From<String> for AccountUuid {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for AccountUuid {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Represents different account providers supported by the launcher
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountProvider {
    /// Microsoft accounts (premium accounts)
    Microsoft,
    /// ElyBy alternative authentication server
    ElyBy,
    /// LittleSkin alternative authentication server
    LittleSkin,
    /// Offline/Cracked accounts (username only, no authentication)
    Offline,
}

impl Display for AccountProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountProvider::Microsoft => write!(f, "Microsoft"),
            AccountProvider::ElyBy => write!(f, "ElyBy"),
            AccountProvider::LittleSkin => write!(f, "LittleSkin"),
            AccountProvider::Offline => write!(f, "Offline"),
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
            AccountProvider::Offline => "Offline",
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

    /// Check if this provider supports username-only authentication (offline)
    pub fn supports_username_only(&self) -> bool {
        matches!(self, AccountProvider::Offline)
    }
}

/// Account information structure with type-safe fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier for the account
    pub uuid: String, // Keep as String for backward compatibility
    /// Username used for login
    pub username: String, // Keep as String for backward compatibility
    /// Display name (may differ from username)
    pub display_name: String,
    /// Account provider
    pub provider: AccountProvider,
    /// Current access token (if available)
    pub access_token: Option<String>, // Keep as String for backward compatibility
    /// Whether the token needs refreshing
    pub needs_refresh: bool,
}

impl Account {
    /// Create a new account with type-safe constructors
    pub fn new(
        uuid: impl Into<String>,
        username: impl Into<String>,
        display_name: impl Into<String>,
        provider: AccountProvider,
        access_token: Option<String>,
        needs_refresh: bool,
    ) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.into(),
            display_name: display_name.into(),
            provider,
            access_token,
            needs_refresh,
        }
    }

    /// Get the UUID as a type-safe wrapper
    pub fn uuid_typed(&self) -> AccountUuid {
        AccountUuid::new(&self.uuid)
    }

    /// Get the username as a type-safe wrapper
    pub fn username_typed(&self) -> Username {
        Username::new_unchecked(&self.username)
    }

    /// Get the access token as a type-safe wrapper
    pub fn access_token_typed(&self) -> Option<AccessToken> {
        self.access_token.as_ref().map(AccessToken::new)
    }

    /// Get a modified username for display purposes
    pub fn display_username(&self) -> String {
        let suffix = match self.provider {
            AccountProvider::Microsoft => "",
            AccountProvider::ElyBy => " (ElyBy)",
            AccountProvider::LittleSkin => " (LittleSkin)",
            AccountProvider::Offline => " (Offline)",
        };
        format!("{}{}", self.display_name, suffix)
    }

    /// Get the authlib URL for this account (if applicable)
    pub fn authlib_url(&self) -> Option<&'static str> {
        match self.provider {
            AccountProvider::Microsoft => None,
            AccountProvider::ElyBy => Some("ely.by"),
            AccountProvider::LittleSkin => Some("https://littleskin.cn/api/yggdrasil"),
            AccountProvider::Offline => None,
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
/// Uses secure strings to prevent password leaks in memory
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct LoginCredentials {
    pub username: String, // Username is not as sensitive, keep as String for compatibility
    pub password: SecureString,
    pub totp_code: Option<SecureString>,
}

impl LoginCredentials {
    /// Create new login credentials with automatic security
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        totp_code: Option<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: SecureString::new(password.into()),
            totp_code: totp_code.map(SecureString::new),
        }
    }

    /// Create credentials from strings (for backward compatibility)
    pub fn from_strings(username: String, password: String, totp_code: Option<String>) -> Self {
        Self::new(username, password, totp_code)
    }

    /// Get the password for authentication (use carefully and clear result)
    pub fn get_password(&self) -> String {
        self.password.expose_secret()
    }

    /// Get the TOTP code for authentication (use carefully and clear result)
    pub fn get_totp_code(&self) -> Option<String> {
        self.totp_code.as_ref().map(|totp| totp.expose_secret())
    }

    /// Get combined password with TOTP (for providers that concatenate them)
    pub fn get_combined_password(&self) -> String {
        let mut combined = self.password.expose_secret();
        if let Some(totp) = &self.totp_code {
            if !totp.is_empty() {
                combined.push(':');
                combined.push_str(&totp.expose_secret());
            }
        }
        combined
    }
}

impl std::fmt::Debug for LoginCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginCredentials")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("totp_code", &self.totp_code.as_ref().map(|_| "<redacted>"))
            .finish()
    }
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
