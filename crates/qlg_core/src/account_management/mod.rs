//! Account Management Module
//!
//! This module provides a unified interface for managing Minecraft accounts
//! across different authentication providers.

mod manager;
mod providers;
mod traits;
mod types;

pub use manager::{AccountManager, KeyringCredentialStore};
pub use providers::*;
pub use traits::{
    AccountManager as AccountManagerTrait, AuthEventHandler, AuthProvider, CredentialStore,
};
pub use types::*;

// Re-export the trait so methods are available
pub use traits::AccountManager as AccountManagerTraitMethods;
