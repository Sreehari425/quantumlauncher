//! # QLG Core Usage Examples
//!
//! This file demonstrates how to use the QLG Core account management system.

//! Tip: dont commit you actual username and password (happend to me)

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

//  Configuration: Set to true to test keyring storage
const ENABLE_KEYRING_STORAGE: bool = true;
const SHOW_STORED_ACCOUNTS: bool = true;

//  Test Configuration: Enable specific provider tests
const TEST_ELYBY_LOGIN: bool = false; // Set to true and add credentials to test
const TEST_MICROSOFT_OAUTH: bool = false; // Set to true to test OAuth flow
const TEST_LITTLESKIN_CREDS: bool = false; // Set to true and add credentials to test
const TEST_LITTLESKIN_OAUTH: bool = false; // Set to true to test OAuth flow
const TEST_OFFLINE_LOGIN: bool = true; // Safe to keep enabled

/// Example: Test keyring storage functionality
pub async fn example_keyring_storage() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🔐 Testing Keyring Storage Functionality...");
    println!();

    if !ENABLE_KEYRING_STORAGE {
        println!("   ℹ️  Keyring storage testing disabled (ENABLE_KEYRING_STORAGE = false)");
        println!();
        return Ok(());
    }

    // First, create an offline account (safe to test with)
    println!("1. Creating test offline account...");
    let result = manager.quick_offline_login("KeyringTestUser").await?;

    match result {
        AuthResult::Success(account) => {
            println!(
                "   ✅ Created: {} (UUID: {})",
                account.display_username(),
                account.uuid
            );

            // Note: Offline accounts don't store refresh tokens since they don't have any
            // But we can test retrieving accounts from keyring
            println!("2. Testing keyring retrieval...");
            match manager.get_accounts().await {
                Ok(accounts) => {
                    println!("   ✅ Retrieved {} accounts from keyring:", accounts.len());
                    for stored_account in &accounts {
                        println!(
                            "     - {} ({}) - UUID: {}",
                            stored_account.display_username(),
                            stored_account.provider,
                            stored_account.uuid
                        );
                        if stored_account.access_token.is_some() {
                            println!("       🔑 Has access token (stored in keyring)");
                        } else {
                            println!("       🔓 No access token (offline account)");
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed to retrieve from keyring: {}", e);
                    println!(
                        "   💡 This might be normal if keyring is not available on this system"
                    );
                }
            }

            // Test checking specific account
            println!("3. Testing specific account lookup...");
            match manager
                .get_account("KeyringTestUser", AccountProvider::Offline)
                .await
            {
                Ok(Some(found_account)) => {
                    println!(
                        "   ✅ Found specific account: {}",
                        found_account.display_username()
                    );
                }
                Ok(None) => {
                    println!("   ℹ️  Account not found in keyring (expected for offline accounts)");
                }
                Err(e) => {
                    println!("   ❌ Error looking up account: {}", e);
                }
            }

            // Test removing from keyring
            println!("4. Testing keyring removal...");
            match manager
                .logout("KeyringTestUser", AccountProvider::Offline)
                .await
            {
                Ok(()) => {
                    println!("   ✅ Account logout completed!");
                }
                Err(e) => {
                    println!("   ❌ Failed to logout: {}", e);
                }
            }

            // Verify removal
            println!("5. Verifying removal...");
            match manager.get_accounts().await {
                Ok(accounts) => {
                    let remaining = accounts
                        .iter()
                        .filter(|acc| acc.display_username() == "KeyringTestUser")
                        .count();
                    if remaining == 0 {
                        println!("   ✅ Account successfully removed from keyring");
                    } else {
                        println!(
                            "   ⚠️  Account still found in keyring ({} instances)",
                            remaining
                        );
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed to verify removal: {}", e);
                }
            }
        }
        _ => {
            println!("   ❌ Failed to create test account");
        }
    }

    println!();
    Ok(())
}

/// Example: Test keyring storage with real authentication
pub async fn example_keyring_with_real_auth() -> Result<(), Box<dyn std::error::Error>> {
    let _manager = AccountManager::new();

    if !ENABLE_KEYRING_STORAGE {
        return Ok(());
    }

    println!("🔑 Testing Keyring with Real Authentication...");
    println!("   💡 This example shows how accounts with tokens are stored");
    println!();

    // You can uncomment and test with real credentials to see token storage
    println!("   ℹ️  To test with real credentials:");
    println!("   1. Uncomment one of the login examples below");
    println!("   2. Add real credentials");
    println!("   3. Run the example to see token storage in action");
    println!();

    /*
    // Example: Test with ElyBy credentials (stores refresh token)
    match manager.quick_login(AccountProvider::ElyBy, "your_username", "your_password").await? {
        AuthResult::Success(account) => {
            println!("   ✅ ElyBy login successful - refresh token stored in keyring!");
            println!("   🔑 Token: {}", account.access_token.as_ref().unwrap_or(&"None".to_string()));

            // Show that it's now in stored accounts
            let accounts = manager.get_accounts().await?;
            println!("   📋 Total stored accounts: {}", accounts.len());
        }
        _ => {}
    }
    */

    println!("   📝 Note: Offline accounts don't store tokens since they don't need them");
    println!("   📝 Real accounts (ElyBy, Microsoft, LittleSkin) store refresh tokens securely");
    println!();

    Ok(())
}

/// Example: Show all stored accounts in keyring
pub async fn example_show_stored_accounts() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AccountManager::new();

    if !SHOW_STORED_ACCOUNTS {
        return Ok(());
    }

    println!("📋 Checking Stored Accounts in Keyring...");

    match manager.get_accounts().await {
        Ok(accounts) => {
            if accounts.is_empty() {
                println!("   📭 No accounts found in keyring");
            } else {
                println!("   📬 Found {} stored accounts:", accounts.len());
                for (i, account) in accounts.iter().enumerate() {
                    println!(
                        "   {}. {} ({}) - UUID: {}",
                        i + 1,
                        account.display_username(),
                        account.provider,
                        account.uuid
                    );

                    // Check if token is still valid
                    if let Some(_token) = &account.access_token {
                        println!("      🔑 Has access token");
                    } else {
                        println!("      🔓 No access token (offline account)");
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to access keyring: {}", e);
            println!("   💡 This might be normal if keyring is not available on this system");
        }
    }

    println!();
    Ok(())
}

/// Example: ElyBy login with username and password
pub async fn example_elyby_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Replace with your actual ElyBy credentials
    let username = "your_email_or_username";
    let password = "your_password";

    println!("Attempting ElyBy login for: {}", username);

    match manager
        .quick_login(AccountProvider::ElyBy, username, password)
        .await?
    {
        AuthResult::Success(account) => {
            println!(
                "✓ ElyBy login successful! Welcome, {}",
                account.display_name
            );
            println!("  UUID: {}", account.uuid);
            println!("  Provider: {}", account.provider);
        }
        AuthResult::RequiresTwoFactor => {
            println!("Two-factor authentication required");
            // You can retry with TOTP code:
            // manager.quick_login_with_2fa(AccountProvider::ElyBy, username, password, "123456").await?;
        }
        AuthResult::Failed(error) => {
            println!("✗ ElyBy login failed: {}", error);
        }
    }

    Ok(())
}

/// Example: LittleSkin credential login (email/username + password)
pub async fn example_littleskin_credentials() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Replace with your actual LittleSkin credentials
    let email_or_username = "your_email"; // or username
    let password = "your_password";

    println!(
        "Attempting LittleSkin credential login for: {}",
        email_or_username
    );

    match manager
        .quick_login(AccountProvider::LittleSkin, email_or_username, password)
        .await?
    {
        AuthResult::Success(account) => {
            println!(
                "✓ LittleSkin credential login successful! Welcome, {}",
                account.display_name
            );
            println!("  UUID: {}", account.uuid);
            println!("  Provider: {}", account.provider);
        }
        AuthResult::RequiresTwoFactor => {
            println!("Two-factor authentication required for LittleSkin");
            // You can retry with TOTP code:
            // manager.quick_login_with_2fa(AccountProvider::LittleSkin, email_or_username, password, "123456").await?;
        }
        AuthResult::Failed(error) => {
            println!("✗ LittleSkin credential login failed: {}", error);
        }
    }

    Ok(())
}

/// Example: LittleSkin OAuth login
pub async fn example_littleskin_oauth() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("Starting LittleSkin OAuth flow...");

    // Start OAuth flow for LittleSkin
    let oauth_flow = manager
        .start_oauth_login(AccountProvider::LittleSkin)
        .await?;

    println!("🌐 Please visit: {}", oauth_flow.verification_uri);
    println!("🔑 Enter code: {}", oauth_flow.user_code);
    println!("⏳ Waiting for authorization...");
    println!();
    println!("📋 Steps to complete:");
    println!("  1. Visit the URL above in your browser");
    println!("  2. Enter the code: {}", oauth_flow.user_code);
    println!("  3. Login with your LittleSkin account");
    println!("  4. Authorize the application");
    println!();
    println!("⏱️  This example will automatically poll for completion...");

    // Poll for completion (device code flow)
    let mut attempts = 0;
    let max_attempts = 60; // 5 minutes with 5-second intervals

    loop {
        attempts += 1;
        if attempts > max_attempts {
            println!("❌ OAuth flow timed out after 5 minutes");
            return Ok(());
        }

        println!(
            "🔄 Checking authorization status... (attempt {}/{})",
            attempts, max_attempts
        );

        // Try to complete the OAuth flow
        match manager
            .complete_oauth_login(AccountProvider::LittleSkin, &oauth_flow.device_code)
            .await
        {
            Ok(AuthResult::Success(account)) => {
                println!(
                    "✅ LittleSkin OAuth login successful! Welcome, {}",
                    account.display_name
                );
                println!("  UUID: {}", account.uuid);
                println!("  Provider: {}", account.provider);
                break;
            }
            Ok(AuthResult::Failed(error)) => {
                let error_str = error.to_string();
                if error_str.contains("authorization_pending") || error_str.contains("slow_down") {
                    // Still waiting for user authorization
                    println!("⏳ Still waiting for authorization... ({})", error_str);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                } else {
                    println!("❌ LittleSkin OAuth login failed: {}", error);
                    break;
                }
            }
            Err(e) => {
                println!("❌ Error during OAuth flow: {}", e);
                break;
            }
            _ => unreachable!("LittleSkin OAuth doesn't use 2FA"),
        }
    }

    Ok(())
}

/// Example: Microsoft OAuth login  
pub async fn example_microsoft_oauth() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("Starting Microsoft OAuth flow...");

    // Start OAuth flow
    let oauth_flow = manager
        .start_oauth_login(AccountProvider::Microsoft)
        .await?;

    println!("🌐 Please visit: {}", oauth_flow.verification_uri);
    println!("🔑 Enter code: {}", oauth_flow.user_code);
    println!("⏳ Waiting for authorization...");
    println!();
    println!("📋 Steps to complete:");
    println!("  1. Visit the URL above in your browser");
    println!("  2. Enter the code: {}", oauth_flow.user_code);
    println!("  3. Login with your Microsoft account");
    println!("  4. Authorize the application");
    println!();
    println!("⏱️  This example will automatically poll for completion...");

    // Poll for completion (device code flow)
    let mut attempts = 0;
    let max_attempts = 60; // 5 minutes with 5-second intervals

    loop {
        attempts += 1;
        if attempts > max_attempts {
            println!("❌ OAuth flow timed out after 5 minutes");
            return Ok(());
        }

        println!(
            "🔄 Checking authorization status... (attempt {}/{})",
            attempts, max_attempts
        );

        // Try to complete the OAuth flow
        match manager
            .complete_oauth_login(AccountProvider::Microsoft, &oauth_flow.device_code)
            .await
        {
            Ok(AuthResult::Success(account)) => {
                println!(
                    "✅ Microsoft OAuth login successful! Welcome, {}",
                    account.display_name
                );
                println!("  UUID: {}", account.uuid);
                println!("  Provider: {}", account.provider);
                break;
            }
            Ok(AuthResult::Failed(error)) => {
                let error_str = error.to_string();
                if error_str.contains("authorization_pending") || error_str.contains("slow_down") {
                    // Still waiting for user authorization
                    println!("⏳ Still waiting for authorization... ({})", error_str);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                } else {
                    println!("❌ Microsoft OAuth login failed: {}", error);
                    break;
                }
            }
            Err(e) => {
                println!("❌ Error during OAuth flow: {}", e);
                break;
            }
            _ => unreachable!("Microsoft OAuth doesn't use 2FA"),
        }
    }

    Ok(())
}

/// Example: Using convenience methods for different providers
pub async fn example_convenience_methods() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🚀 Testing convenience methods...");

    // Test different quick login methods
    println!("1. Quick login with ElyBy:");
    match manager
        .quick_login(AccountProvider::ElyBy, "test_username", "test_password")
        .await?
    {
        AuthResult::Success(account) => {
            println!("   ✓ Success: {}", account.display_username());
        }
        AuthResult::Failed(error) => {
            println!("   ✗ Failed: {} (expected with dummy credentials)", error);
        }
        _ => {}
    }

    println!("2. Quick offline login:");
    match manager.quick_offline_login("TestPlayer").await? {
        AuthResult::Success(account) => {
            println!(
                "   ✓ Success: {} (UUID: {})",
                account.display_username(),
                account.uuid
            );
        }
        _ => {}
    }

    println!("3. Provider capabilities check:");
    for provider in [
        AccountProvider::Microsoft,
        AccountProvider::ElyBy,
        AccountProvider::LittleSkin,
        AccountProvider::Offline,
    ] {
        let (creds, oauth, username) = manager.provider_capabilities(provider)?;
        println!(
            "   {}: creds={}, oauth={}, username={}",
            provider, creds, oauth, username
        );
    }

    Ok(())
}

/// Example: Account management
pub async fn example_account_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Get all stored accounts
    let accounts = manager.get_accounts().await?;
    println!("Found {} stored accounts", accounts.len());

    for account in accounts {
        println!("- {} ({})", account.display_username(), account.provider);

        // Ensure token is valid
        let refreshed_account = manager.ensure_valid_token(&account).await?;
        if refreshed_account.access_token.is_some() {
            println!("  ✓ Token is valid");
        } else {
            println!("  ✗ Token needs refresh");
        }
    }

    // Check what providers are available
    let providers = manager.supported_providers();
    println!("Supported providers: {:?}", providers);

    // Check provider capabilities
    for provider in providers {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        println!(
            "{}: credentials={}, oauth={}, username_only={}",
            provider, creds, oauth, username_only
        );
    }

    Ok(())
}

/// Example: Offline (username-only) login
pub async fn example_offline_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Simple offline login with just username
    match manager.quick_offline_login("TestPlayer").await? {
        AuthResult::Success(account) => {
            println!("Offline login successful: {}", account.display_username());
            println!("UUID: {}", account.uuid);
            println!("Access token: {:?}", account.access_token);
        }
        AuthResult::Failed(error) => {
            println!("Offline login failed: {}", error);
        }
        _ => unreachable!("Offline login doesn't use 2FA"),
    }

    // You can also use the longer form
    match manager
        .login_username_only(AccountProvider::Offline, "AnotherPlayer")
        .await?
    {
        AuthResult::Success(account) => {
            println!(
                "Username-only login successful: {}",
                account.display_username()
            );
        }
        _ => {}
    }

    Ok(())
}

/// Example: Logout
pub async fn example_logout() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Logout a specific account
    manager.logout("username", AccountProvider::ElyBy).await?;
    println!("Account logged out successfully");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core Account Management Examples");
    println!("=====================================");
    println!();

    /*
    🔧 HOW TO TEST EACH PROVIDER:

    1. 🔵 ElyBy (Credentials only):
       - Set TEST_ELYBY_LOGIN = true at the top of this file
       - Replace "your_email_or_username" and "your_password" in example_elyby_login()

    2. 🟢 Microsoft (OAuth only):
       - Set TEST_MICROSOFT_OAUTH = true at the top of this file
       - Follow the OAuth flow when prompted

    3. 🟡 LittleSkin (Both methods):
       - For credentials: Set TEST_LITTLESKIN_CREDS = true
       - Replace credentials in example_littleskin_credentials()
       - For OAuth: Set TEST_LITTLESKIN_OAUTH = true

    4. ⚫ Offline (Username-only):
       - TEST_OFFLINE_LOGIN = true (enabled by default)
       - No credentials required

    🔐 KEYRING STORAGE:
    - ENABLE_KEYRING_STORAGE = true (tests credential storage)
    - SHOW_STORED_ACCOUNTS = true (shows accounts in keyring)
    */

    // Show all available provider capabilities first
    let manager = AccountManager::new();
    println!("📋 Supported Providers and Their Capabilities:");
    for provider in manager.supported_providers() {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        println!(
            "  {} - Credentials: {}, OAuth: {}, Username-Only: {}",
            provider, creds, oauth, username_only
        );
    }
    println!();

    // Show current stored accounts (if any)
    if SHOW_STORED_ACCOUNTS {
        example_show_stored_accounts().await?;
    }

    // Test keyring storage functionality
    if ENABLE_KEYRING_STORAGE {
        example_keyring_storage().await?;
        example_keyring_with_real_auth().await?;
    }

    // Test each provider based on configuration
    if TEST_ELYBY_LOGIN {
        println!("🔵 Testing ElyBy Login...");
        example_elyby_login().await?;
        println!();
    }

    if TEST_MICROSOFT_OAUTH {
        println!("🟢 Testing Microsoft OAuth...");
        example_microsoft_oauth().await?;
        println!();
    }

    if TEST_LITTLESKIN_CREDS {
        println!("🟡 Testing LittleSkin Credential Login...");
        example_littleskin_credentials().await?;
        println!();
    }

    if TEST_LITTLESKIN_OAUTH {
        println!("🟡 Testing LittleSkin OAuth...");
        example_littleskin_oauth().await?;
        println!();
    }

    if TEST_OFFLINE_LOGIN {
        println!("⚫ Testing Offline Login...");
        example_offline_login().await?;
        println!();
    }

    // 🔧 Other examples (uncomment to test)
    // example_convenience_methods().await?;
    // example_account_management().await?;
    // example_logout().await?;

    println!("✅ Examples completed!");
    println!();
    println!("💡 Configuration Tips:");
    println!("   - Edit the constants at the top to enable/disable tests");
    println!("   - Set ENABLE_KEYRING_STORAGE = true to test credential storage");
    println!("   - Set TEST_* flags = true and add real credentials to test providers");
    println!("   - Keyring storage works automatically when you login with real accounts!");
    Ok(())
}
