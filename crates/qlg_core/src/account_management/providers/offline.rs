//! Offline account provider for username-only authentication

use crate::account_management::{errors::*, traits::AuthProvider, types::*};
use async_trait::async_trait;

/// Offline account provider for username-only authentication (cracked accounts)
pub struct OfflineProvider;

impl OfflineProvider {
    pub fn new() -> Self {
        Self
    }

    /// Generate a simple UUID for offline accounts
    /// Use the same format as the main launcher for consistency
    fn generate_offline_uuid(_username: &str) -> String {
        // Use the same simple UUID that the main launcher uses for offline accounts
        // No need for complex generation - this is just for offline/cracked accounts
        "00000000-0000-0000-0000-000000000000".to_string()
    }
}

impl Default for OfflineProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for OfflineProvider {
    fn provider_type(&self) -> AccountProvider {
        AccountProvider::Offline
    }

    async fn login_with_credentials(&self, _credentials: &LoginCredentials) -> Result<AuthResult> {
        // Offline accounts don't use credentials, only username
        Err(AccountError::UnsupportedProvider)
    }

    async fn login_with_username(&self, username: &str) -> Result<AuthResult> {
        // Validate username using the new type-safe wrapper
        let validated_username = match Username::new(username) {
            Ok(u) => u,
            Err(e) => return Err(AccountError::InvalidInput(e)),
        };

        // Validate offline-specific username format (basic Minecraft username rules)
        let username_str = validated_username.as_str();
        if username_str.len() < 3 || username_str.len() > 16 {
            return Err(AccountError::InvalidInput(
                "Offline username must be between 3 and 16 characters".to_string(),
            ));
        }

        // Check for invalid characters (Minecraft usernames should be alphanumeric + underscore)
        if !username_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AccountError::InvalidInput(
                "Offline username can only contain letters, numbers, and underscores".to_string(),
            ));
        }

        let account = Account::new(
            Self::generate_offline_uuid(username_str),
            username_str,
            username_str, // For offline accounts, display name = username
            AccountProvider::Offline,
            None, // No access token for offline accounts
            false, // Offline accounts don't need refresh
        );

        Ok(AuthResult::Success(account))
    }

    async fn start_oauth_flow(&self) -> Result<OAuthFlow> {
        // Offline accounts don't support OAuth
        Err(AccountError::UnsupportedProvider)
    }

    async fn complete_oauth(&self, _device_code: &str) -> Result<AuthResult> {
        // Offline accounts don't support OAuth
        Err(AccountError::UnsupportedProvider)
    }

    async fn refresh_token(&self, account: &Account) -> Result<Account> {
        // Offline accounts don't have tokens to refresh
        // Just return the account as-is
        Ok(account.clone())
    }

    async fn validate_credentials(&self, _credentials: &LoginCredentials) -> Result<bool> {
        // Offline accounts don't use credentials
        Err(AccountError::UnsupportedProvider)
    }

    fn supports_credentials_auth(&self) -> bool {
        false
    }

    fn supports_oauth_auth(&self) -> bool {
        false
    }

    fn supports_username_only_auth(&self) -> bool {
        true
    }
}
