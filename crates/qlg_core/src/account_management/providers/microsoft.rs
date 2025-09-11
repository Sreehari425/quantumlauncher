//! Microsoft account authentication provider

use crate::account_management::{errors::*, traits::AuthProvider, types::*};
use async_trait::async_trait;
use ql_instances::auth::ms;
use ql_instances::auth::AccountType;

/// Microsoft account authentication provider
pub struct MicrosoftProvider;

impl MicrosoftProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MicrosoftProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for MicrosoftProvider {
    fn provider_type(&self) -> AccountProvider {
        AccountProvider::Microsoft
    }

    async fn login_with_credentials(&self, _credentials: &LoginCredentials) -> Result<AuthResult> {
        // Microsoft doesn't support direct credential login
        Err(AccountError::UnsupportedProvider)
    }

    async fn login_with_username(&self, _username: &str) -> Result<AuthResult> {
        // Microsoft doesn't support username-only login
        Err(AccountError::UnsupportedProvider)
    }

    async fn start_oauth_flow(&self) -> Result<OAuthFlow> {
        let auth_response = ms::login_1_link()
            .await
            .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;

        Ok(OAuthFlow {
            verification_uri: auth_response.verification_uri,
            user_code: auth_response.user_code,
            device_code: auth_response.device_code,
            expires_in: auth_response.expires_in as u64,
            interval: auth_response.interval as u64,
        })
    }

    async fn complete_oauth(&self, device_code: &str) -> Result<AuthResult> {
        // Create a mock AuthCodeResponse with the device code
        let auth_code_response = ms::AuthCodeResponse {
            user_code: String::new(), // Not needed for completion
            device_code: device_code.to_string(),
            verification_uri: String::new(), // Not needed for completion
            expires_in: 0,                   // Not needed for completion
            interval: 0,                     // Not needed for completion
            message: String::new(),          // Not needed for completion
        };

        // Wait for user authorization
        let auth_token_response = ms::login_2_wait(auth_code_response)
            .await
            .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;

        // Complete Xbox authentication
        let account_data = ms::login_3_xbox(auth_token_response, None, true)
            .await
            .map_err(|e| match e {
                ql_instances::auth::ms::Error::DoesntOwnGame => AccountError::NoMinecraftOwnership,
                _ => AccountError::AuthenticationFailed(e.to_string()),
            })?;

        // Convert to our Account type
        let account = Account {
            uuid: account_data.uuid,
            username: account_data.username,
            display_name: account_data.nice_username,
            provider: AccountProvider::Microsoft,
            access_token: account_data.access_token,
            needs_refresh: account_data.needs_refresh,
        };

        Ok(AuthResult::Success(account))
    }

    async fn refresh_token(&self, account: &Account) -> Result<Account> {
        // Get stored refresh token
        let refresh_token =
            ql_instances::auth::read_refresh_token(&account.username, AccountType::Microsoft)
                .map_err(|e| AccountError::Keyring(e.0))?;

        // Refresh the token
        let account_data = ms::login_refresh(account.username.clone(), refresh_token, None)
            .await
            .map_err(|e| AccountError::AuthenticationFailed(e.to_string()))?;

        // Convert back to our Account type
        Ok(Account {
            uuid: account_data.uuid,
            username: account_data.username,
            display_name: account_data.nice_username,
            provider: AccountProvider::Microsoft,
            access_token: account_data.access_token,
            needs_refresh: account_data.needs_refresh,
        })
    }

    async fn validate_credentials(&self, _credentials: &LoginCredentials) -> Result<bool> {
        // Microsoft doesn't support credential validation without full login
        Err(AccountError::UnsupportedProvider)
    }

    fn supports_credentials_auth(&self) -> bool {
        false
    }

    fn supports_oauth_auth(&self) -> bool {
        true
    }

    fn supports_username_only_auth(&self) -> bool {
        false
    }
}
