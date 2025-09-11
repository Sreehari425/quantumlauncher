//! Integration tests for QLG Core

use qlg_core::{AccountManager, AccountProvider, AccountManagerTrait, AuthResult, Username, AccessToken, AccountUuid};

#[tokio::test]
async fn test_type_safe_wrappers() {
    // Test Username validation
    let valid_username = Username::new("TestUser123").unwrap();
    assert_eq!(valid_username.as_str(), "TestUser123");
    
    // Test invalid usernames
    assert!(Username::new("").is_err()); // Empty
    assert!(Username::new("ab").is_err()); // Too short
    assert!(Username::new("a".repeat(51)).is_err()); // Too long
    
    // Test AccessToken
    let token = AccessToken::new("test_token_123");
    assert_eq!(token.as_str(), "test_token_123");
    assert!(!token.is_empty());
    assert_eq!(token.preview(), "test_tok...");
    
    let empty_token = AccessToken::new("");
    assert!(empty_token.is_empty());
    
    // Test AccountUuid
    let uuid = AccountUuid::new("123e4567-e89b-12d3-a456-426614174000");
    assert_eq!(uuid.as_str(), "123e4567-e89b-12d3-a456-426614174000");
    
    let offline_uuid = AccountUuid::offline();
    assert_eq!(offline_uuid.as_str(), "00000000-0000-0000-0000-000000000000");
}

#[tokio::test]
async fn test_enhanced_offline_validation() {
    let mut manager = AccountManager::new();
    
    // Test valid offline usernames
    let result = manager.quick_offline_login("TestUser").await.unwrap();
    if let AuthResult::Success(account) = result {
        assert_eq!(account.username, "TestUser");
        assert_eq!(account.provider, AccountProvider::Offline);
    } else {
        panic!("Expected successful offline login");
    }
    
    // Test invalid usernames
    let result = manager.quick_offline_login("ab").await; // Too short
    assert!(result.is_err());
    
    let result = manager.quick_offline_login("toolongusernamehere").await; // Too long
    assert!(result.is_err());
    
    let result = manager.quick_offline_login("user@domain.com").await; // Invalid chars
    assert!(result.is_err());
}

#[tokio::test]
async fn test_account_type_safety() {
    let mut manager = AccountManager::new();
    
    let result = manager.quick_offline_login("TestUser").await.unwrap();
    if let AuthResult::Success(account) = result {
        // Test type-safe accessors
        let uuid_typed = account.uuid_typed();
        assert_eq!(uuid_typed.as_str(), "00000000-0000-0000-0000-000000000000");
        
        let username_typed = account.username_typed();
        assert_eq!(username_typed.as_str(), "TestUser");
        
        let token_typed = account.access_token_typed();
        assert!(token_typed.is_none()); // Offline accounts don't have tokens
    }
}

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
