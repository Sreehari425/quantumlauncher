//! Integration tests for QLG Core

use qlg_core::{AccountManager, AccountProvider, AccountManagerTrait, AuthResult};

#[tokio::test]
async fn test_account_manager_creation() {
    let manager = AccountManager::new();
    
    // Check that all providers are available
    assert!(manager.is_provider_available(AccountProvider::Microsoft));
    assert!(manager.is_provider_available(AccountProvider::ElyBy));
    assert!(manager.is_provider_available(AccountProvider::LittleSkin));
    assert!(manager.is_provider_available(AccountProvider::Offline));
}

#[tokio::test]
async fn test_provider_capabilities() {
    let manager = AccountManager::new();
    
    // Microsoft: OAuth only
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::Microsoft).unwrap();
    assert!(!creds); // No credential auth
    assert!(oauth);  // Has OAuth auth
    assert!(!username_only); // No username-only auth
    
    // ElyBy: Credentials only
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::ElyBy).unwrap();
    assert!(creds);  // Has credential auth
    assert!(!oauth); // No OAuth auth
    assert!(!username_only); // No username-only auth
    
    // LittleSkin: Both credentials and OAuth
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::LittleSkin).unwrap();
    assert!(creds); // Has credential auth
    assert!(oauth); // Has OAuth auth
    assert!(!username_only); // No username-only auth
    
    // Offline: Username only
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::Offline).unwrap();
    assert!(!creds); // No credential auth
    assert!(!oauth); // No OAuth auth
    assert!(username_only); // Has username-only auth
}

#[tokio::test]
async fn test_supported_providers() {
    let manager = AccountManager::new();
    let providers = manager.supported_providers();
    
    assert_eq!(providers.len(), 4);
    assert!(providers.contains(&AccountProvider::Microsoft));
    assert!(providers.contains(&AccountProvider::ElyBy));
    assert!(providers.contains(&AccountProvider::LittleSkin));
    assert!(providers.contains(&AccountProvider::Offline));
}

#[tokio::test]
async fn test_account_provider_display() {
    assert_eq!(AccountProvider::Microsoft.to_string(), "Microsoft");
    assert_eq!(AccountProvider::ElyBy.to_string(), "ElyBy");
    assert_eq!(AccountProvider::LittleSkin.to_string(), "LittleSkin");
    assert_eq!(AccountProvider::Offline.to_string(), "Offline");
}

#[tokio::test]
async fn test_account_provider_properties() {
    // Microsoft requires ownership
    assert!(AccountProvider::Microsoft.requires_ownership());
    assert!(!AccountProvider::ElyBy.requires_ownership());
    assert!(!AccountProvider::LittleSkin.requires_ownership());
    assert!(!AccountProvider::Offline.requires_ownership());
    
    // OAuth support
    assert!(AccountProvider::Microsoft.supports_oauth());
    assert!(!AccountProvider::ElyBy.supports_oauth());
    assert!(AccountProvider::LittleSkin.supports_oauth());
    assert!(!AccountProvider::Offline.supports_oauth());
    
    // Credentials support
    assert!(!AccountProvider::Microsoft.supports_credentials());
    assert!(AccountProvider::ElyBy.supports_credentials());
    assert!(AccountProvider::LittleSkin.supports_credentials());
    assert!(!AccountProvider::Offline.supports_credentials());
    
    // Username-only support
    assert!(!AccountProvider::Microsoft.supports_username_only());
    assert!(!AccountProvider::ElyBy.supports_username_only());
    assert!(!AccountProvider::LittleSkin.supports_username_only());
    assert!(AccountProvider::Offline.supports_username_only());
}

#[tokio::test]
async fn test_offline_login() {
    let mut manager = AccountManager::new();
    
    // Test valid offline login
    let result = manager.quick_offline_login("TestPlayer").await.unwrap();
    match result {
        AuthResult::Success(account) => {
            assert_eq!(account.username, "TestPlayer");
            assert_eq!(account.display_name, "TestPlayer");
            assert_eq!(account.provider, AccountProvider::Offline);
            assert!(account.access_token.is_none());
            assert!(!account.needs_refresh);
            assert!(!account.uuid.is_empty());
        }
        _ => panic!("Expected successful offline login"),
    }
    
    // Test invalid usernames
    assert!(manager.quick_offline_login("").await.is_err());
    assert!(manager.quick_offline_login("ab").await.is_err()); // Too short
    assert!(manager.quick_offline_login("this_username_is_way_too_long").await.is_err()); // Too long
    assert!(manager.quick_offline_login("Test@Player").await.is_err()); // Invalid characters
}
