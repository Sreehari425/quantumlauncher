//! Main account manager implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::account_management::{
    traits::{AccountManager as AccountManagerTrait, AuthProvider, CredentialStore},
    types::*,
    providers::*,
};

/// Default credential store using the system keyring
pub struct KeyringCredentialStore;

#[async_trait]
impl CredentialStore for KeyringCredentialStore {
    async fn store_refresh_token(&self, username: &str, provider: AccountProvider, token: &str) -> Result<()> {
        // We need to access the keyring entry differently since get_keyring_entry is private
        // For now, we'll use the keyring directly with the same format as the original code
        let keyring_name = format!(
            "{}{}",
            username,
            match provider {
                AccountProvider::Microsoft => "",
                AccountProvider::ElyBy => "#elyby", 
                AccountProvider::LittleSkin => "#littleskin",
            }
        );
        
        let entry = keyring::Entry::new("QuantumLauncher", &keyring_name)
            .map_err(AccountError::Keyring)?;
        
        entry.set_password(token)
            .map_err(AccountError::Keyring)?;
        
        Ok(())
    }
    
    async fn get_refresh_token(&self, username: &str, provider: AccountProvider) -> Result<String> {
        let account_type = match provider {
            AccountProvider::Microsoft => ql_instances::auth::AccountType::Microsoft,
            AccountProvider::ElyBy => ql_instances::auth::AccountType::ElyBy,
            AccountProvider::LittleSkin => ql_instances::auth::AccountType::LittleSkin,
        };
        
        ql_instances::auth::read_refresh_token(username, account_type)
            .map_err(|e| AccountError::Keyring(e.0))
    }
    
    async fn remove_credentials(&self, username: &str, provider: AccountProvider) -> Result<()> {
        let account_type = match provider {
            AccountProvider::Microsoft => ql_instances::auth::AccountType::Microsoft,
            AccountProvider::ElyBy => ql_instances::auth::AccountType::ElyBy,
            AccountProvider::LittleSkin => ql_instances::auth::AccountType::LittleSkin,
        };
        
        ql_instances::auth::logout(username, account_type)
            .map_err(|e| AccountError::AuthenticationFailed(e))?;
        
        Ok(())
    }
    
    async fn has_credentials(&self, username: &str, provider: AccountProvider) -> Result<bool> {
        match self.get_refresh_token(username, provider).await {
            Ok(_) => Ok(true),
            Err(AccountError::Keyring(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    async fn list_accounts(&self) -> Result<Vec<(String, AccountProvider)>> {
        // This is a simplified implementation
        // In practice, you might want to maintain a separate registry of accounts
        // For now, we'll return an empty list as the keyring doesn't provide enumeration
        Ok(vec![])
    }
}

/// Main account manager that provides a unified interface for account operations
pub struct AccountManager {
    providers: HashMap<AccountProvider, Arc<dyn AuthProvider>>,
    credential_store: Arc<dyn CredentialStore>,
}

impl AccountManager {
    /// Create a new account manager with default providers and credential store
    pub fn new() -> Self {
        let mut providers: HashMap<AccountProvider, Arc<dyn AuthProvider>> = HashMap::new();
        
        providers.insert(
            AccountProvider::Microsoft,
            Arc::new(MicrosoftProvider::new()),
        );
        providers.insert(
            AccountProvider::ElyBy,
            Arc::new(ElyByProvider::new()),
        );
        providers.insert(
            AccountProvider::LittleSkin,
            Arc::new(LittleSkinProvider::new()),
        );
        
        Self {
            providers,
            credential_store: Arc::new(KeyringCredentialStore),
        }
    }
    
    /// Create account manager with custom credential store
    pub fn with_credential_store(credential_store: Arc<dyn CredentialStore>) -> Self {
        let mut providers: HashMap<AccountProvider, Arc<dyn AuthProvider>> = HashMap::new();
        
        providers.insert(
            AccountProvider::Microsoft,
            Arc::new(MicrosoftProvider::new()),
        );
        providers.insert(
            AccountProvider::ElyBy,
            Arc::new(ElyByProvider::new()),
        );
        providers.insert(
            AccountProvider::LittleSkin,
            Arc::new(LittleSkinProvider::new()),
        );
        
        Self {
            providers,
            credential_store,
        }
    }
    
    /// Get a specific provider
    fn get_provider(&self, provider: AccountProvider) -> Result<&Arc<dyn AuthProvider>> {
        self.providers
            .get(&provider)
            .ok_or(AccountError::UnsupportedProvider)
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountManagerTrait for AccountManager {
    async fn login(&mut self, provider: AccountProvider, credentials: &LoginCredentials) -> Result<AuthResult> {
        let provider_impl = self.get_provider(provider)?;
        
        if !provider_impl.supports_credentials_auth() {
            return Err(AccountError::UnsupportedProvider);
        }
        
        provider_impl.login_with_credentials(credentials).await
    }
    
    async fn start_oauth_login(&mut self, provider: AccountProvider) -> Result<OAuthFlow> {
        let provider_impl = self.get_provider(provider)?;
        
        if !provider_impl.supports_oauth_auth() {
            return Err(AccountError::UnsupportedProvider);
        }
        
        provider_impl.start_oauth_flow().await
    }
    
    async fn complete_oauth_login(&mut self, provider: AccountProvider, device_code: &str) -> Result<AuthResult> {
        let provider_impl = self.get_provider(provider)?;
        
        if !provider_impl.supports_oauth_auth() {
            return Err(AccountError::UnsupportedProvider);
        }
        
        provider_impl.complete_oauth(device_code).await
    }
    
    async fn refresh_account(&mut self, account: &Account) -> Result<Account> {
        let provider_impl = self.get_provider(account.provider)?;
        provider_impl.refresh_token(account).await
    }
    
    async fn logout(&mut self, username: &str, provider: AccountProvider) -> Result<()> {
        self.credential_store.remove_credentials(username, provider).await
    }
    
    async fn get_accounts(&self) -> Result<Vec<Account>> {
        let stored_accounts = self.credential_store.list_accounts().await?;
        let mut accounts = Vec::new();
        
        for (username, provider) in stored_accounts {
            if let Ok(Some(account)) = self.get_account(&username, provider).await {
                accounts.push(account);
            }
        }
        
        Ok(accounts)
    }
    
    async fn get_account(&self, username: &str, provider: AccountProvider) -> Result<Option<Account>> {
        // Check if we have stored credentials for this account
        if !self.credential_store.has_credentials(username, provider).await? {
            return Ok(None);
        }
        
        // For now, create a basic account structure
        // In a real implementation, you might want to store additional metadata
        Ok(Some(Account {
            uuid: String::new(), // Will be filled when token is refreshed
            username: username.to_string(),
            display_name: username.to_string(),
            provider,
            access_token: None,
            needs_refresh: true,
        }))
    }
    
    async fn ensure_valid_token(&mut self, account: &Account) -> Result<Account> {
        if account.needs_refresh || account.access_token.is_none() {
            self.refresh_account(account).await
        } else {
            Ok(account.clone())
        }
    }
    
    fn is_provider_available(&self, provider: AccountProvider) -> bool {
        self.providers.contains_key(&provider)
    }
}

// Convenience functions for easy usage
impl AccountManager {
    /// Quick login with username and password
    pub async fn quick_login(&mut self, provider: AccountProvider, username: &str, password: &str) -> Result<AuthResult> {
        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
            totp_code: None,
        };
        
        self.login(provider, &credentials).await
    }
    
    /// Quick login with username, password, and 2FA code
    pub async fn quick_login_with_2fa(&mut self, provider: AccountProvider, username: &str, password: &str, totp: &str) -> Result<AuthResult> {
        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
            totp_code: Some(totp.to_string()),
        };
        
        self.login(provider, &credentials).await
    }
    
    /// Get list of supported providers
    pub fn supported_providers(&self) -> Vec<AccountProvider> {
        self.providers.keys().copied().collect()
    }
    
    /// Check what authentication methods a provider supports
    pub fn provider_capabilities(&self, provider: AccountProvider) -> Result<(bool, bool)> {
        let provider_impl = self.get_provider(provider)?;
        Ok((
            provider_impl.supports_credentials_auth(),
            provider_impl.supports_oauth_auth(),
        ))
    }
}
