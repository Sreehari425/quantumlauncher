//! Offline account provider for username-only authentication

use async_trait::async_trait;
use uuid::Uuid;
use crate::account_management::{traits::AuthProvider, types::*};

/// Offline account provider for username-only authentication (cracked accounts)
pub struct OfflineProvider;

impl OfflineProvider {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate a UUID for offline accounts based on username
    /// This creates a deterministic UUID based on the username
    fn generate_offline_uuid(username: &str) -> String {
        // Create a deterministic UUID based on username
        // This ensures the same username always gets the same UUID
        let namespace = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let uuid = Uuid::new_v5(&namespace, username.as_bytes());
        uuid.to_string()
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
        if username.trim().is_empty() {
            return Err(AccountError::AuthenticationFailed("Username cannot be empty".to_string()));
        }
        
        // Validate username format (basic Minecraft username rules)
        if username.len() < 3 || username.len() > 16 {
            return Err(AccountError::AuthenticationFailed(
                "Username must be between 3 and 16 characters".to_string()
            ));
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AccountError::AuthenticationFailed(
                "Username can only contain letters, numbers, and underscores".to_string()
            ));
        }
        
        // Create offline account
        let account = Account {
            uuid: Self::generate_offline_uuid(username),
            username: username.to_string(),
            display_name: username.to_string(),
            provider: AccountProvider::Offline,
            access_token: None, // Offline accounts don't have access tokens
            needs_refresh: false, // No tokens to refresh
        };
        
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
