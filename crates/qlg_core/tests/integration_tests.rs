//! Integration tests for QLG Core

use qlg_core::{AccountManager, AccountProvider, AccountManagerTrait};

#[tokio::test]
async fn test_account_manager_creation() {
    let manager = AccountManager::new();
    
    // Check that all providers are available
    assert!(manager.is_provider_available(AccountProvider::Microsoft));
    assert!(manager.is_provider_available(AccountProvider::ElyBy));
    assert!(manager.is_provider_available(AccountProvider::LittleSkin));
}

#[tokio::test]
async fn test_provider_capabilities() {
    let manager = AccountManager::new();
    
    // Microsoft: OAuth only
    let (creds, oauth) = manager.provider_capabilities(AccountProvider::Microsoft).unwrap();
    assert!(!creds); // No credential auth
    assert!(oauth);  // Has OAuth auth
    
    // ElyBy: Credentials only
    let (creds, oauth) = manager.provider_capabilities(AccountProvider::ElyBy).unwrap();
    assert!(creds);  // Has credential auth
    assert!(!oauth); // No OAuth auth
    
    // LittleSkin: Both
    let (creds, oauth) = manager.provider_capabilities(AccountProvider::LittleSkin).unwrap();
    assert!(creds); // Has credential auth
    assert!(oauth); // Has OAuth auth
}

#[tokio::test]
async fn test_supported_providers() {
    let manager = AccountManager::new();
    let providers = manager.supported_providers();
    
    assert_eq!(providers.len(), 3);
    assert!(providers.contains(&AccountProvider::Microsoft));
    assert!(providers.contains(&AccountProvider::ElyBy));
    assert!(providers.contains(&AccountProvider::LittleSkin));
}

#[tokio::test]
async fn test_account_provider_display() {
    assert_eq!(AccountProvider::Microsoft.to_string(), "Microsoft");
    assert_eq!(AccountProvider::ElyBy.to_string(), "ElyBy");
    assert_eq!(AccountProvider::LittleSkin.to_string(), "LittleSkin");
}

#[tokio::test]
async fn test_account_provider_properties() {
    // Microsoft requires ownership
    assert!(AccountProvider::Microsoft.requires_ownership());
    assert!(!AccountProvider::ElyBy.requires_ownership());
    assert!(!AccountProvider::LittleSkin.requires_ownership());
    
    // OAuth support
    assert!(AccountProvider::Microsoft.supports_oauth());
    assert!(!AccountProvider::ElyBy.supports_oauth());
    assert!(AccountProvider::LittleSkin.supports_oauth());
    
    // Credentials support
    assert!(!AccountProvider::Microsoft.supports_credentials());
    assert!(AccountProvider::ElyBy.supports_credentials());
    assert!(AccountProvider::LittleSkin.supports_credentials());
}
