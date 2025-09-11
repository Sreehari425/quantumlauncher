//! Account Management Module
//!
//! This module provides a unified interface for managing Minecraft accounts
//! across different authentication providers.

mod errors;
mod manager;
mod providers;
mod traits;
mod types;

// Re-export main types and traits
pub use errors::{AccountError, Result};
pub use manager::{AccountManager, KeyringCredentialStore};
pub use providers::*;
pub use traits::{
    AccountManager as AccountManagerTrait, AuthEventHandler, AuthProvider, CredentialStore,
};
pub use types::*;
