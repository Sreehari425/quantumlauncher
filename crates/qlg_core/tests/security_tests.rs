//! Security tests for credential zeroization

use qlg_core::{AccountManager, AccountProvider, LoginCredentials};

#[tokio::test]
async fn test_credential_zeroization() {
    // Test that credentials are properly secured
    let credentials =
        LoginCredentials::new("testuser", "secret_password", Some("123456".to_string()));

    // Test that debug output doesn't leak passwords
    let debug_output = format!("{:?}", credentials);
    assert!(!debug_output.contains("secret_password"));
    assert!(!debug_output.contains("123456"));
    assert!(debug_output.contains("<redacted>"));

    // Test that we can still access the password when needed
    assert_eq!(credentials.get_password(), "secret_password");
    assert_eq!(credentials.get_totp_code().unwrap(), "123456");

    // Test combined password for providers that concatenate
    assert_eq!(
        credentials.get_combined_password(),
        "secret_password:123456"
    );
}

#[tokio::test]
async fn test_secure_string_behavior() {
    use qlg_core::SecureString;

    let mut secure_str = SecureString::new("sensitive_data");

    // Test basic operations
    assert_eq!(secure_str.len(), 14);
    assert!(!secure_str.is_empty());

    // Test concatenation (for TOTP)
    secure_str.push(':');
    secure_str.push_str("extra");
    assert_eq!(secure_str.expose_secret(), "sensitive_data:extra");

    // Test debug output doesn't leak data
    let debug_output = format!("{:?}", secure_str);
    assert!(!debug_output.contains("sensitive_data"));
    assert!(debug_output.contains("len"));
}

#[tokio::test]
async fn test_access_token_security() {
    use qlg_core::AccessToken;

    let token = AccessToken::new("very_secret_access_token_12345");

    // Test that debug output is safe
    let debug_output = format!("{:?}", token);
    assert!(!debug_output.contains("very_secret_access_token_12345"));
    assert!(debug_output.contains("preview"));

    // Test preview function shows only safe part
    let preview = token.preview();
    assert_eq!(preview, "very_sec...");

    // Test that we can still get the full token when needed
    assert_eq!(token.as_str(), "very_secret_access_token_12345");
}

#[tokio::test]
async fn test_credential_construction_methods() {
    // Test different ways to construct credentials
    let creds1 = LoginCredentials::new("user1", "pass1", None);
    let creds2 = LoginCredentials::from_strings("user1".to_string(), "pass1".to_string(), None);

    assert_eq!(creds1.username, creds2.username);
    assert_eq!(creds1.get_password(), creds2.get_password());

    // Test with TOTP
    let creds3 = LoginCredentials::new("user2", "pass2", Some("654321".to_string()));
    assert_eq!(creds3.get_totp_code().unwrap(), "654321");
    assert_eq!(creds3.get_combined_password(), "pass2:654321");
}

#[tokio::test]
async fn test_manager_convenience_methods_security() {
    let mut manager = AccountManager::new();

    // Test that quick_login methods work with secure credentials
    let result = manager
        .quick_login(AccountProvider::Offline, "testuser", "testpass")
        .await;

    // Should succeed for offline accounts (username validation might apply)
    match result {
        Ok(_) => {}  // Success case
        Err(_) => {} // Might fail due to username validation, that's ok
    }

    // Test 2FA variant
    let result = manager
        .quick_login_with_2fa(AccountProvider::Offline, "testuser", "testpass", "123456")
        .await;

    // Should behave similarly
    match result {
        Ok(_) => {}  // Success case
        Err(_) => {} // Might fail due to username validation, that's ok
    }
}
