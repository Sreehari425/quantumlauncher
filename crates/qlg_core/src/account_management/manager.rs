//! Main account manager implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::account_management::{
    errors::*,
    providers::*,
    traits::{AccountManager as AccountManagerTrait, AuthProvider, CredentialStore},
    types::*,
};

/// Simplified config structure for reading launcher accounts
#[derive(Serialize, Deserialize, Debug)]
struct LauncherConfig {
    pub accounts: Option<HashMap<String, ConfigAccount>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ConfigAccount {
    pub uuid: String,
    pub account_type: Option<String>,
    pub keyring_identifier: Option<String>,
    pub username_nice: Option<String>,
}

/// Default credential store using the system keyring
pub struct KeyringCredentialStore;

#[async_trait]
impl CredentialStore for KeyringCredentialStore {
    async fn store_refresh_token(
        &self,
        username: &str,
        provider: AccountProvider,
        token: &str,
    ) -> Result<()> {
        // Offline accounts don't store tokens
        if matches!(provider, AccountProvider::Offline) {
            return Ok(());
        }

        // We need to access the keyring entry differently since get_keyring_entry is private
        // For now, we'll use the keyring directly with the same format as the original code
        let keyring_name = format!(
            "{}{}",
            username,
            match provider {
                AccountProvider::Microsoft => "",
                AccountProvider::ElyBy => "#elyby",
                AccountProvider::LittleSkin => "#littleskin",
                AccountProvider::Offline => unreachable!(), // Handled above
            }
        );

        let entry =
            keyring::Entry::new("QuantumLauncher", &keyring_name).map_err(AccountError::Keyring)?;

        entry.set_password(token).map_err(AccountError::Keyring)?;

        Ok(())
    }

    async fn get_refresh_token(&self, username: &str, provider: AccountProvider) -> Result<String> {
        // Get stored authentication token from keyring.
        // Note: Despite the method name, this returns access tokens for Yggdrasil providers
        // and refresh tokens for Microsoft (as per ql_instances auth implementation).
        // Offline accounts don't have tokens
        if matches!(provider, AccountProvider::Offline) {
            return Err(AccountError::AccountNotFound);
        }

        let account_type = match provider {
            AccountProvider::Microsoft => ql_instances::auth::AccountType::Microsoft,
            AccountProvider::ElyBy => ql_instances::auth::AccountType::ElyBy,
            AccountProvider::LittleSkin => ql_instances::auth::AccountType::LittleSkin,
            AccountProvider::Offline => unreachable!(), // Handled above
        };

        ql_instances::auth::read_refresh_token(username, account_type)
            .map_err(|e| AccountError::Keyring(e.0))
    }

    async fn remove_credentials(&self, username: &str, provider: AccountProvider) -> Result<()> {
        // Offline accounts don't have stored credentials to remove
        if matches!(provider, AccountProvider::Offline) {
            return Ok(());
        }

        let account_type = match provider {
            AccountProvider::Microsoft => ql_instances::auth::AccountType::Microsoft,
            AccountProvider::ElyBy => ql_instances::auth::AccountType::ElyBy,
            AccountProvider::LittleSkin => ql_instances::auth::AccountType::LittleSkin,
            AccountProvider::Offline => unreachable!(), // Handled above
        };

        ql_instances::auth::logout(username, account_type)
            .map_err(|e| AccountError::AuthenticationFailed(e))?;

        Ok(())
    }

    async fn has_credentials(&self, username: &str, provider: AccountProvider) -> Result<bool> {
        // Offline accounts don't have stored credentials
        if matches!(provider, AccountProvider::Offline) {
            return Ok(false);
        }

        match self.get_refresh_token(username, provider).await {
            Ok(_) => Ok(true),
            Err(AccountError::Keyring(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn list_accounts(&self) -> Result<Vec<(String, AccountProvider)>> {
        // Try to read accounts from the main launcher's config file
        match self.read_launcher_config().await {
            Ok(accounts) => Ok(accounts),
            Err(_) => {
                // If we can't read the launcher config, return empty list
                // This means qlg_core is running independently
                Ok(vec![])
            }
        }
    }
}

impl KeyringCredentialStore {
    /// Read accounts from the main launcher's config.json file
    async fn read_launcher_config(&self) -> Result<Vec<(String, AccountProvider)>> {
        // Try to get the launcher directory from ql_core
        let launcher_dir = match ql_core::file_utils::get_launcher_dir() {
            Ok(dir) => dir,
            Err(_) => return Ok(vec![]), // Can't find launcher dir
        };

        let config_path = launcher_dir.join("config.json");
        if !config_path.exists() {
            return Ok(vec![]); // No config file exists
        }

        // Read and parse the config file
        let config_content = match tokio::fs::read_to_string(&config_path).await {
            Ok(content) => content,
            Err(_) => return Ok(vec![]), // Can't read config
        };

        let config: LauncherConfig = match serde_json::from_str(&config_content) {
            Ok(config) => config,
            Err(_) => return Ok(vec![]), // Invalid JSON
        };

        let Some(accounts) = config.accounts else {
            return Ok(vec![]); // No accounts in config
        };

        let mut result = Vec::new();
        for (username, account) in accounts {
            // Convert account type string to AccountProvider enum
            let provider = match account.account_type.as_deref() {
                Some("Microsoft") => AccountProvider::Microsoft,
                Some("ElyBy") => AccountProvider::ElyBy,
                Some("LittleSkin") => AccountProvider::LittleSkin,
                _ => continue, // Skip unknown account types
            };

            // Use keyring_identifier if available, otherwise use username
            let keyring_username = account.keyring_identifier.unwrap_or(username);
            result.push((keyring_username, provider));
        }

        Ok(result)
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
        providers.insert(AccountProvider::ElyBy, Arc::new(ElyByProvider::new()));
        providers.insert(
            AccountProvider::LittleSkin,
            Arc::new(LittleSkinProvider::new()),
        );
        providers.insert(AccountProvider::Offline, Arc::new(OfflineProvider::new()));

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
        providers.insert(AccountProvider::ElyBy, Arc::new(ElyByProvider::new()));
        providers.insert(
            AccountProvider::LittleSkin,
            Arc::new(LittleSkinProvider::new()),
        );
        providers.insert(AccountProvider::Offline, Arc::new(OfflineProvider::new()));

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
    async fn login(
        &mut self,
        provider: AccountProvider,
        credentials: &LoginCredentials,
    ) -> Result<AuthResult> {
        let provider_impl = self.get_provider(provider)?;

        if !provider_impl.supports_credentials_auth() {
            return Err(AccountError::UnsupportedProvider);
        }

        provider_impl.login_with_credentials(credentials).await
    }

    async fn login_username_only(
        &mut self,
        provider: AccountProvider,
        username: &str,
    ) -> Result<AuthResult> {
        let provider_impl = self.get_provider(provider)?;

        if !provider_impl.supports_username_only_auth() {
            return Err(AccountError::UnsupportedProvider);
        }

        provider_impl.login_with_username(username).await
    }

    async fn start_oauth_login(&mut self, provider: AccountProvider) -> Result<OAuthFlow> {
        let provider_impl = self.get_provider(provider)?;

        if !provider_impl.supports_oauth_auth() {
            return Err(AccountError::UnsupportedProvider);
        }

        provider_impl.start_oauth_flow().await
    }

    async fn complete_oauth_login(
        &mut self,
        provider: AccountProvider,
        device_code: &str,
    ) -> Result<AuthResult> {
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
        self.credential_store
            .remove_credentials(username, provider)
            .await
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

    async fn get_account(
        &self,
        username: &str,
        provider: AccountProvider,
    ) -> Result<Option<Account>> {
        // Check if we have stored credentials for this account
        if !self
            .credential_store
            .has_credentials(username, provider)
            .await?
        {
            return Ok(None);
        }

        // Try to get account details from launcher config
        let (uuid, display_name) = match self
            .get_account_details_from_config(username, provider)
            .await
        {
            Ok(Some((uuid, display_name))) => (uuid, display_name),
            _ => {
                // Fallback: use username as UUID and display name
                (String::new(), username.to_string())
            }
        };

        // Try to get the stored token (access token for Yggdrasil providers, refresh token for Microsoft)
        let access_token = if matches!(provider, AccountProvider::Offline) {
            None // Offline accounts don't have tokens
        } else {
            // Try to get the token from keyring
            match self
                .credential_store
                .get_refresh_token(username, provider)
                .await
            {
                Ok(token) => Some(token),
                Err(_) => None, // Token might be expired or not available
            }
        };

        let needs_refresh = access_token.is_none() && !matches!(provider, AccountProvider::Offline);

        Ok(Some(Account {
            uuid,
            username: username.to_string(),
            display_name,
            provider,
            access_token,
            needs_refresh,
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
    pub async fn quick_login(
        &mut self,
        provider: AccountProvider,
        username: &str,
        password: &str,
    ) -> Result<AuthResult> {
        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
            totp_code: None,
        };

        self.login(provider, &credentials).await
    }

    /// Quick login with username, password, and 2FA code
    pub async fn quick_login_with_2fa(
        &mut self,
        provider: AccountProvider,
        username: &str,
        password: &str,
        totp: &str,
    ) -> Result<AuthResult> {
        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
            totp_code: Some(totp.to_string()),
        };

        self.login(provider, &credentials).await
    }

    /// Quick offline login with just username (for cracked accounts)
    pub async fn quick_offline_login(&mut self, username: &str) -> Result<AuthResult> {
        self.login_username_only(AccountProvider::Offline, username)
            .await
    }

    /// Get list of supported providers
    pub fn supported_providers(&self) -> Vec<AccountProvider> {
        self.providers.keys().copied().collect()
    }

    /// Check what authentication methods a provider supports
    /// Returns (credentials_auth, oauth_auth, username_only_auth)
    pub fn provider_capabilities(&self, provider: AccountProvider) -> Result<(bool, bool, bool)> {
        let provider_impl = self.get_provider(provider)?;
        Ok((
            provider_impl.supports_credentials_auth(),
            provider_impl.supports_oauth_auth(),
            provider_impl.supports_username_only_auth(),
        ))
    }

    /// Get account details (UUID, display name) from launcher config
    async fn get_account_details_from_config(
        &self,
        username: &str,
        provider: AccountProvider,
    ) -> Result<Option<(String, String)>> {
        let launcher_dir = match ql_core::file_utils::get_launcher_dir() {
            Ok(dir) => dir,
            Err(_) => return Ok(None),
        };

        let config_path = launcher_dir.join("config.json");
        if !config_path.exists() {
            return Ok(None);
        }

        let config_content = match std::fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };

        let config: LauncherConfig = match serde_json::from_str(&config_content) {
            Ok(config) => config,
            Err(_) => return Ok(None),
        };

        let Some(accounts) = config.accounts else {
            return Ok(None);
        };

        // Look for matching account
        for (config_username, account) in accounts {
            let account_provider = match account.account_type.as_deref() {
                Some("Microsoft") => AccountProvider::Microsoft,
                Some("ElyBy") => AccountProvider::ElyBy,
                Some("LittleSkin") => AccountProvider::LittleSkin,
                _ => continue,
            };

            // Check if this matches our search criteria
            let keyring_username = account
                .keyring_identifier
                .as_ref()
                .unwrap_or(&config_username);
            if keyring_username == username && account_provider == provider {
                let display_name = account.username_nice.unwrap_or(config_username);
                return Ok(Some((account.uuid, display_name)));
            }
        }

        Ok(None)
    }
}
