//! Core traits for account management

use async_trait::async_trait;
use super::types::*;

/// Core trait for account authentication providers
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> super::types::AccountProvider;
    
    /// Authenticate using username and password
    async fn login_with_credentials(&self, credentials: &LoginCredentials) -> Result<AuthResult>;
    
    /// Start OAuth authentication flow (if supported)
    async fn start_oauth_flow(&self) -> Result<OAuthFlow>;
    
    /// Complete OAuth authentication using device code
    async fn complete_oauth(&self, device_code: &str) -> Result<AuthResult>;
    
    /// Refresh an existing account's token
    async fn refresh_token(&self, account: &Account) -> Result<Account>;
    
    /// Check if credentials are valid without full login
    async fn validate_credentials(&self, credentials: &LoginCredentials) -> Result<bool>;
    
    /// Check if the provider supports specific authentication method
    fn supports_credentials_auth(&self) -> bool;
    fn supports_oauth_auth(&self) -> bool;
}

/// Trait for secure credential storage
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Store refresh token for an account
    async fn store_refresh_token(&self, username: &str, provider: super::types::AccountProvider, token: &str) -> Result<()>;
    
    /// Retrieve refresh token for an account
    async fn get_refresh_token(&self, username: &str, provider: super::types::AccountProvider) -> Result<String>;
    
    /// Remove stored credentials for an account
    async fn remove_credentials(&self, username: &str, provider: super::types::AccountProvider) -> Result<()>;
    
    /// Check if credentials exist for an account
    async fn has_credentials(&self, username: &str, provider: super::types::AccountProvider) -> Result<bool>;
    
    /// List all stored accounts
    async fn list_accounts(&self) -> Result<Vec<(String, super::types::AccountProvider)>>;
}

/// Main account manager trait that orchestrates authentication
#[async_trait]
pub trait AccountManager: Send + Sync {
    /// Login with username and password
    async fn login(&mut self, provider: super::types::AccountProvider, credentials: &LoginCredentials) -> Result<AuthResult>;
    
    /// Start OAuth login flow
    async fn start_oauth_login(&mut self, provider: super::types::AccountProvider) -> Result<OAuthFlow>;
    
    /// Complete OAuth login
    async fn complete_oauth_login(&mut self, provider: super::types::AccountProvider, device_code: &str) -> Result<AuthResult>;
    
    /// Refresh an account's token
    async fn refresh_account(&mut self, account: &Account) -> Result<Account>;
    
    /// Logout an account (remove stored credentials)
    async fn logout(&mut self, username: &str, provider: super::types::AccountProvider) -> Result<()>;
    
    /// Get all stored accounts
    async fn get_accounts(&self) -> Result<Vec<Account>>;
    
    /// Get a specific account by username and provider
    async fn get_account(&self, username: &str, provider: super::types::AccountProvider) -> Result<Option<Account>>;
    
    /// Auto-refresh account if needed
    async fn ensure_valid_token(&mut self, account: &Account) -> Result<Account>;
    
    /// Check if a provider is available and configured
    fn is_provider_available(&self, provider: super::types::AccountProvider) -> bool;
}

/// Trait for handling authentication events (optional, for UI integration)
pub trait AuthEventHandler: Send + Sync {
    /// Called when authentication starts
    fn on_auth_started(&self, provider: super::types::AccountProvider);
    
    /// Called when authentication progresses
    fn on_auth_progress(&self, message: &str, progress: Option<(usize, usize)>);
    
    /// Called when authentication completes successfully
    fn on_auth_success(&self, account: &Account);
    
    /// Called when authentication fails
    fn on_auth_failed(&self, error: &AccountError);
    
    /// Called when two-factor authentication is required
    fn on_two_factor_required(&self);
}
