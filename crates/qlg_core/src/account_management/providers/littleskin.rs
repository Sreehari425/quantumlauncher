//! LittleSkin account authentication provider

use async_trait::async_trait;
use ql_instances::auth::yggdrasil;
use ql_instances::auth::AccountType;
use crate::account_management::{traits::AuthProvider, types::*};

/// LittleSkin account authentication provider
pub struct LittleSkinProvider;

impl LittleSkinProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LittleSkinProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for LittleSkinProvider {
    fn provider_type(&self) -> AccountProvider {
        AccountProvider::LittleSkin
    }
    
    async fn login_with_credentials(&self, credentials: &LoginCredentials) -> Result<AuthResult> {
        // Prepare password with TOTP if provided
        let mut password = credentials.password.clone();
        if let Some(ref totp) = credentials.totp_code {
            if !totp.is_empty() {
                password.push(':');
                password.push_str(totp);
            }
        }
        
        // Attempt login
        let result = yggdrasil::login_new(
            credentials.username.clone(),
            password,
            AccountType::LittleSkin,
        )
        .await
        .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;
        
        match result {
            yggdrasil::Account::Account(account_data) => {
                let account = Account {
                    uuid: account_data.uuid,
                    username: account_data.username,
                    display_name: account_data.nice_username,
                    provider: AccountProvider::LittleSkin,
                    access_token: account_data.access_token,
                    needs_refresh: account_data.needs_refresh,
                };
                Ok(AuthResult::Success(account))
            }
            yggdrasil::Account::NeedsOTP => {
                Ok(AuthResult::RequiresTwoFactor)
            }
        }
    }
    
    async fn login_with_username(&self, _username: &str) -> Result<AuthResult> {
        // LittleSkin doesn't support username-only login
        Err(AccountError::UnsupportedProvider)
    }
    
    async fn start_oauth_flow(&self) -> Result<OAuthFlow> {
        // Start LittleSkin OAuth flow
        let oauth_data = yggdrasil::oauth::request_device_code()
            .await
            .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;
        
        Ok(OAuthFlow {
            verification_uri: oauth_data.verification_uri,
            user_code: oauth_data.user_code,
            device_code: oauth_data.device_code,
            expires_in: oauth_data.expires_in,
            interval: oauth_data.interval,
        })
    }
    
    async fn complete_oauth(&self, device_code: &str) -> Result<AuthResult> {
        // Extract interval and expires_in from device code flow
        // Note: In a real implementation, you'd store these values from start_oauth_flow
        // For now, we'll use reasonable defaults
        let interval = 5; // 5 seconds
        let expires_in = 900; // 15 minutes
        
        let result = yggdrasil::oauth::poll_device_token(
            device_code.to_string(),
            interval,
            expires_in,
        )
        .await
        .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;
        
        match result {
            yggdrasil::Account::Account(account_data) => {
                let account = Account {
                    uuid: account_data.uuid,
                    username: account_data.username,
                    display_name: account_data.nice_username,
                    provider: AccountProvider::LittleSkin,
                    access_token: account_data.access_token,
                    needs_refresh: account_data.needs_refresh,
                };
                Ok(AuthResult::Success(account))
            }
            yggdrasil::Account::NeedsOTP => {
                Ok(AuthResult::RequiresTwoFactor)
            }
        }
    }
    
    async fn refresh_token(&self, account: &Account) -> Result<Account> {
        // Get stored refresh token
        let refresh_token = ql_instances::auth::read_refresh_token(&account.username, AccountType::LittleSkin)
            .map_err(|e| AccountError::Keyring(e.0))?;
        
        // Refresh the token
        let account_data = yggdrasil::login_refresh(
            account.username.clone(),
            refresh_token,
            AccountType::LittleSkin,
        )
        .await
        .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;
        
        // Convert back to our Account type
        Ok(Account {
            uuid: account_data.uuid,
            username: account_data.username,
            display_name: account_data.nice_username,
            provider: AccountProvider::LittleSkin,
            access_token: account_data.access_token,
            needs_refresh: account_data.needs_refresh,
        })
    }
    
    async fn validate_credentials(&self, credentials: &LoginCredentials) -> Result<bool> {
        // For LittleSkin, we can try to login and see if it succeeds
        // This is a basic validation - in practice you might want a lighter check
        match self.login_with_credentials(credentials).await {
            Ok(AuthResult::Success(_)) => Ok(true),
            Ok(AuthResult::RequiresTwoFactor) => Ok(true), // Credentials are valid, just need 2FA
            Ok(AuthResult::Failed(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }
    
    fn supports_credentials_auth(&self) -> bool {
        true
    }
    
    fn supports_oauth_auth(&self) -> bool {
        true
    }
    
    fn supports_username_only_auth(&self) -> bool {
        false
    }
}
