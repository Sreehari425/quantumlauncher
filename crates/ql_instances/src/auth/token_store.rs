//! Token storage abstraction layer.
//!
//! Provides a unified interface for storing and retrieving authentication tokens,
//! supporting both system keyring and encrypted file storage backends.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

use super::{encrypted_store, AccountType, KeyringError};

/// Global setting for which storage method to use.
/// 0 = Keyring, 1 = EncryptedFile
static STORAGE_METHOD: AtomicU8 = AtomicU8::new(0);

/// The method used for storing authentication tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TokenStorageMethod {
    #[default]
    #[serde(rename = "keyring")]
    Keyring,
    #[serde(rename = "encrypted_file")]
    EncryptedFile,
}

impl TokenStorageMethod {
    /// Set the global storage method.
    pub fn set_global(self) {
        STORAGE_METHOD.store(self as u8, Ordering::SeqCst);
    }

    /// Get the current global storage method.
    #[must_use]
    pub fn get_global() -> Self {
        match STORAGE_METHOD.load(Ordering::SeqCst) {
            1 => TokenStorageMethod::EncryptedFile,
            _ => TokenStorageMethod::Keyring,
        }
    }
}

/// Errors that can occur during token storage operations.
#[derive(Debug, thiserror::Error)]
pub enum TokenStoreError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] KeyringError),

    #[error("Encrypted store error: {0}")]
    EncryptedStore(#[from] encrypted_store::EncryptedStoreError),
}

/// Generate the storage key for a username and account type.
fn make_storage_key(username: &str, account_type: AccountType) -> String {
    format!(
        "{username}{}",
        match account_type {
            AccountType::Microsoft => "",
            AccountType::ElyBy => "#elyby",
            AccountType::LittleSkin => "#littleskin",
        }
    )
}

/// Store a token using the current storage method.
pub fn store_token(
    username: &str,
    token: &str,
    account_type: AccountType,
) -> Result<(), TokenStoreError> {
    let storage_key = make_storage_key(username, account_type);

    match TokenStorageMethod::get_global() {
        TokenStorageMethod::Keyring => {
            let entry = keyring::Entry::new("QuantumLauncher", &storage_key)?;
            entry.set_password(token)?;
            Ok(())
        }
        TokenStorageMethod::EncryptedFile => {
            encrypted_store::store_token(&storage_key, token)?;
            Ok(())
        }
    }
}

/// Read a token using the current storage method.
pub fn read_token(username: &str, account_type: AccountType) -> Result<String, TokenStoreError> {
    let storage_key = make_storage_key(username, account_type);

    match TokenStorageMethod::get_global() {
        TokenStorageMethod::Keyring => {
            let entry = keyring::Entry::new("QuantumLauncher", &storage_key)?;
            let token = entry.get_password()?;
            Ok(token)
        }
        TokenStorageMethod::EncryptedFile => {
            let token = encrypted_store::read_token(&storage_key)?;
            Ok(token)
        }
    }
}

/// Delete a token using the current storage method.
pub fn delete_token(username: &str, account_type: AccountType) -> Result<(), TokenStoreError> {
    let storage_key = make_storage_key(username, account_type);

    match TokenStorageMethod::get_global() {
        TokenStorageMethod::Keyring => {
            let entry = keyring::Entry::new("QuantumLauncher", &storage_key)?;
            // Ignore errors when deleting (credential might not exist)
            let _ = entry.delete_credential();
            Ok(())
        }
        TokenStorageMethod::EncryptedFile => {
            // Ignore errors when deleting (token might not exist)
            let _ = encrypted_store::delete_token(&storage_key);
            Ok(())
        }
    }
}

/// Check if the encrypted store needs to be unlocked.
///
/// Returns `true` if using encrypted storage and it's not yet unlocked.
#[must_use]
pub fn needs_unlock() -> bool {
    TokenStorageMethod::get_global() == TokenStorageMethod::EncryptedFile
        && !encrypted_store::is_unlocked()
}

/// Check if the encrypted store file exists.
///
/// Returns `true` if using encrypted storage and the file exists.
#[must_use]
pub fn encrypted_file_exists() -> bool {
    encrypted_store::encrypted_file_exists()
}

impl From<keyring::Error> for TokenStoreError {
    fn from(err: keyring::Error) -> Self {
        TokenStoreError::Keyring(KeyringError(err))
    }
}
