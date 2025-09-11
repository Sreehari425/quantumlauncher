//! # QLG Core Usage Examples
//!
//! This file demonstrates how to use the QLG Core account management system.
//!
//! ## 🚪 **Logout Function Parameters Guide**
//!
//! The logout function requires two parameters: `username` and `account_type`
//!
//! ### Function Signature:
//! ```rust
//! manager.logout(username: &str, provider: AccountProvider).await?;
//! ```
//!
//! ### Parameter Requirements by Provider:
//!
//! #### 🔵 **ElyBy Accounts**
//! - **Username**: Your ElyBy username (not email)
//! - **Provider**: `AccountProvider::ElyBy`
//! - **Example**: `manager.logout("MyElyByUser", AccountProvider::ElyBy).await?;`
//!
//! #### 🟢 **Microsoft Accounts**  
//! - **Username**: Your Microsoft email address or username
//! - **Provider**: `AccountProvider::Microsoft`
//! - **Example**: `manager.logout("player@outlook.com", AccountProvider::Microsoft).await?;`
//!
//! #### 🟡 **LittleSkin Accounts**
//! - **Username**: Your LittleSkin username
//! - **Provider**: `AccountProvider::LittleSkin`
//! - **Example**: `manager.logout("MyLittleSkinUser", AccountProvider::LittleSkin).await?;`
//!
//! #### ⚫ **Offline Accounts**
//! - **Username**: Any username you created the offline account with
//! - **Provider**: `AccountProvider::Offline`
//! - **Example**: `manager.logout("TestPlayer", AccountProvider::Offline).await?;`
//!
//! ### 📋 **How to Find the Correct Username:**
//! 1. Use `manager.get_accounts().await?` to list all stored accounts
//! 2. Check `account.display_username()` for the exact string to use
//! 3. Use that exact string in the logout function
//!
//! ### ⚠️ **Important Notes:**
//! - Logout removes stored tokens from the system keyring
//! - Users will need to login again after logout
//! - Microsoft accounts will need to complete OAuth flow again
//! - ElyBy/LittleSkin accounts will need to enter credentials again
//! - Offline accounts can be recreated anytime (no stored tokens)

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

//  Advanced Examples: Enable to test logout and token refresh
const TEST_LOGOUT_EXAMPLES: bool = true; // Test logout functionality
const TEST_TOKEN_REFRESH: bool = true; // Test token refresh functionality

/// Example: Test keyring storage functionality
///
/// Note: The keyring stores different token types depending on the provider:
/// - ElyBy & LittleSkin: Access tokens (used directly for authentication)
/// - Microsoft: Refresh tokens (used to get fresh access tokens)
/// - Offline: No tokens (UUID-only authentication)
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

/// Example: Logout functionality
///
/// Demonstrates how to logout accounts and remove stored tokens.
/// This function shows the proper way to logout accounts by account type and username.
///
/// ## Parameters Required for Logout:
/// - `username`: The exact username as stored in the keyring
/// - `account_type`: The provider type (Microsoft, ElyBy, LittleSkin, Offline)
///
/// ## Important Notes:
/// - For **Microsoft** accounts: Use the email address or Microsoft username
/// - For **ElyBy** accounts: Use the ElyBy username (not email)
/// - For **LittleSkin** accounts: Use the LittleSkin username  
/// - For **Offline** accounts: Use any username you created the account with
///
/// ## Finding the Correct Username:
/// 1. Check `manager.get_accounts()` to see stored usernames
/// 2. Look at your launcher's account list
/// 3. Use the exact string shown in `account.display_username()`
pub async fn example_logout() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🔐 Testing Logout Functionality...");
    println!();

    // First, let's see what accounts are available
    let accounts = manager.get_accounts().await?;

    if accounts.is_empty() {
        println!("   ℹ️  No accounts found in keyring to logout");
        println!("   💡 Try logging in with real credentials first to test logout");
        return Ok(());
    }

    println!("   📋 Found {} accounts before logout:", accounts.len());
    for account in &accounts {
        println!(
            "     - {} ({})",
            account.display_username(),
            account.provider
        );
    }
    println!();

    // Test creating and then logging out a test account
    println!("   🔧 Creating test offline account for logout demo...");
    let test_result = manager.quick_offline_login("LogoutTestUser").await?;
    let _test_account = match test_result {
        AuthResult::Success(account) => {
            println!(
                "     ✅ Created: {} ({})",
                account.display_username(),
                account.provider
            );
            account
        }
        _ => return Err("Failed to create test account".into()),
    };

    // Now logout the test account
    println!("   🚪 Logging out test account...");
    manager
        .logout("LogoutTestUser", AccountProvider::Offline)
        .await?;
    println!("     ✅ Test account logged out successfully");

    // Verify it's removed
    let accounts_after = manager.get_accounts().await?;
    println!(
        "   📋 Found {} accounts after logout:",
        accounts_after.len()
    );

    // Demonstrate the interactive logout function
    println!();
    println!("   🔧 Testing Interactive Logout Function...");

    // Create another test account to demonstrate interactive logout
    println!("   📝 Creating another test account for interactive demo...");
    let interactive_test = manager.quick_offline_login("TestUser2").await?;
    if let AuthResult::Success(account) = interactive_test {
        println!(
            "     ✅ Created: {} ({})",
            account.display_username(),
            account.provider
        );

        // Test the interactive logout (without confirmation for demo)
        let logout_success = interactive_logout_account(
            &mut manager,
            "TestUser2",
            AccountProvider::Offline,
            false, // Skip confirmation for demo
        )
        .await?;

        if logout_success {
            println!("   ✅ Interactive logout completed successfully");
        }
    }

    // Demonstrate how to get the correct username for real accounts
    println!();
    println!("   📋 **Real Account Username Reference**:");
    let real_accounts = manager.get_accounts().await?;
    if !real_accounts.is_empty() {
        println!("   ℹ️  For your stored accounts, use these usernames:");
        for acc in real_accounts {
            println!(
                "      - Provider: {} | Username for logout: \"{}\"",
                acc.provider, acc.username
            );
            println!(
                "        Example: manager.logout(\"{}\", AccountProvider::{}).await?;",
                acc.username, acc.provider
            );
        }
    } else {
        println!(
            "   ℹ️  No stored accounts found. After logging in, check this section for examples."
        );
    }

    // Example of logging out real accounts (commented for safety)
    println!();
    println!("   💡 How to logout specific accounts:");
    println!();

    // Show how to logout each type with real examples
    demonstration_logout_examples().await?;

    println!();
    println!("   ⚠️  Warning: This will remove stored tokens from keyring!");
    println!("   ⚠️  Users will need to login again after logout!");

    Ok(())
}

/// Demonstrates the correct way to logout accounts with different providers
/// This function shows the exact parameters needed for each account type
async fn demonstration_logout_examples() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AccountManager::new();

    // Get current accounts to show real examples
    let accounts = manager.get_accounts().await?;

    println!("   📖 Logout Examples by Account Type:");
    println!();

    // Show examples for each provider type
    println!("   🔵 **ElyBy Accounts**:");
    println!("      manager.logout(\"username\", AccountProvider::ElyBy).await?;");
    if let Some(elyby_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::ElyBy))
    {
        println!(
            "      // Real example: manager.logout(\"{}\", AccountProvider::ElyBy).await?;",
            elyby_account.username
        );
    }
    println!();

    println!("   🟢 **Microsoft Accounts**:");
    println!("      manager.logout(\"email@example.com\", AccountProvider::Microsoft).await?;");
    if let Some(ms_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::Microsoft))
    {
        println!(
            "      // Real example: manager.logout(\"{}\", AccountProvider::Microsoft).await?;",
            ms_account.username
        );
    } else {
        println!("      // Example: manager.logout(\"player@outlook.com\", AccountProvider::Microsoft).await?;");
    }
    println!();

    println!("   🟡 **LittleSkin Accounts**:");
    println!("      manager.logout(\"username\", AccountProvider::LittleSkin).await?;");
    if let Some(ls_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::LittleSkin))
    {
        println!(
            "      // Real example: manager.logout(\"{}\", AccountProvider::LittleSkin).await?;",
            ls_account.username
        );
    }
    println!();

    println!("   ⚫ **Offline Accounts**:");
    println!("      manager.logout(\"TestPlayer\", AccountProvider::Offline).await?;");
    println!("      // Note: Offline accounts don't store tokens, logout just removes from memory");
    println!();

    // Interactive logout example (commented for safety)
    println!("   🔧 **Interactive Logout Function**:");
    println!("      Use `interactive_logout_account()` to safely logout with user confirmation");

    Ok(())
}

/// Interactive logout function with safety checks and user confirmation
/// This is the recommended way to implement logout in real applications
pub async fn interactive_logout_account(
    manager: &mut AccountManager,
    username: &str,
    provider: AccountProvider,
    confirm_logout: bool, // Set to false for testing, true for real usage
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("🚪 **Interactive Logout: {} ({})**", username, provider);

    // Step 1: Verify the account exists
    let accounts = manager.get_accounts().await?;
    let account_exists = accounts
        .iter()
        .any(|acc| acc.username == username && acc.provider == provider);

    if !account_exists {
        println!("   ❌ Account not found: {} ({})", username, provider);
        println!("   💡 Available accounts:");
        for acc in accounts {
            println!(
                "      - {} ({}) [username: \"{}\"]",
                acc.display_username(),
                acc.provider,
                acc.username
            );
        }
        return Ok(false);
    }

    // Step 2: Show what will be removed
    println!("   ℹ️  Account found: {} ({})", username, provider);
    match provider {
        AccountProvider::Microsoft => {
            println!("   🔑 This will remove: Refresh token (used to get new access tokens)");
            println!("   ⚠️  You'll need to complete OAuth flow again to re-login");
        }
        AccountProvider::ElyBy | AccountProvider::LittleSkin => {
            println!("   🔑 This will remove: Access token (used for authentication)");
            println!("   ⚠️  You'll need to enter credentials again to re-login");
        }
        AccountProvider::Offline => {
            println!("   📝 This will remove: Account from memory (no tokens stored)");
            println!("   ℹ️  You can create offline accounts again anytime");
        }
    }

    // Step 3: Confirmation (in real apps, ask user)
    if confirm_logout {
        println!("   ❓ Are you sure you want to logout? (This is where you'd ask user)");
        // In real implementation: get user input here
        println!("   ✅ User confirmed logout (simulated)");
    } else {
        println!("   ⚠️  Skipping confirmation for demo (confirm_logout = false)");
    }

    // Step 4: Perform logout
    println!("   🔄 Logging out...");
    match manager.logout(username, provider).await {
        Ok(()) => {
            println!("   ✅ Successfully logged out: {} ({})", username, provider);
            println!("   🧹 Tokens removed from keyring");
            Ok(true)
        }
        Err(e) => {
            println!("   ❌ Logout failed: {}", e);
            println!("   💡 This might mean the account was already logged out");
            Err(e.into())
        }
    }
}

/// Example: Token refresh functionality
/// Demonstrates how to check and refresh expired tokens
pub async fn example_token_refresh() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🔄 Testing Token Refresh Functionality...");
    println!();

    // Get existing accounts
    let accounts = manager.get_accounts().await?;

    if accounts.is_empty() {
        println!("   ℹ️  No accounts with tokens found for refresh testing");
        println!("   💡 This example works best with real logged-in accounts");

        // Demonstrate with offline account (which doesn't need refresh)
        println!();
        println!("   🔧 Creating offline account to demonstrate API...");
        let offline_result = manager.quick_offline_login("RefreshTestUser").await?;
        let offline_account = match offline_result {
            AuthResult::Success(account) => account,
            _ => return Err("Failed to create offline account".into()),
        };

        println!("   🔍 Checking if token needs refresh...");
        let refreshed = manager.ensure_valid_token(&offline_account).await?;

        if refreshed.needs_refresh {
            println!("     ⚠️  Token needs refresh");
        } else {
            println!("     ✅ Token is valid (offline accounts don't need refresh)");
        }

        // Clean up
        manager
            .logout("RefreshTestUser", AccountProvider::Offline)
            .await?;
        return Ok(());
    }

    println!(
        "   📋 Testing token refresh for {} accounts:",
        accounts.len()
    );
    println!();

    for account in accounts {
        println!(
            "   🔍 Testing: {} ({})",
            account.display_username(),
            account.provider
        );

        // Check current token status
        if account.access_token.is_some() {
            println!("     🔑 Has stored token");
        } else {
            println!("     🔓 No token stored");
        }

        if account.needs_refresh {
            println!("     ⚠️  Token marked as needing refresh");
        } else {
            println!("     ✅ Token appears valid");
        }

        // Use ensure_valid_token to refresh if needed
        println!("     🔄 Ensuring token is valid...");
        match manager.ensure_valid_token(&account).await {
            Ok(refreshed_account) => {
                if refreshed_account.access_token.is_some() {
                    println!("     ✅ Token is valid/refreshed");

                    // Show token info (first few chars only for security)
                    if let Some(token) = &refreshed_account.access_token {
                        let preview = if token.len() > 8 {
                            format!("{}...", &token[..8])
                        } else {
                            "***".to_string()
                        };
                        println!("     🔑 Token preview: {}", preview);
                    }
                } else {
                    println!("     ❌ No valid token available");
                }
            }
            Err(e) => {
                println!("     ❌ Token refresh failed: {}", e);
                println!("     💡 This might mean the refresh token is expired");
                println!("     💡 User may need to login again");
            }
        }
        println!();
    }

    println!("   💡 Token Refresh Tips:");
    println!("     - Microsoft accounts: Refresh tokens are used to get new access tokens");
    println!("     - ElyBy/LittleSkin: Access tokens are used directly (may not need refresh)");
    println!("     - Offline accounts: No tokens needed");
    println!("     - Failed refresh usually means user needs to login again");

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

    if TEST_LOGOUT_EXAMPLES {
        println!("🚪 Testing Logout Functionality...");
        example_logout().await?;
        println!();
    }

    if TEST_TOKEN_REFRESH {
        println!("🔄 Testing Token Refresh...");
        example_token_refresh().await?;
        println!();
    }

    // 🔧 Other examples (uncomment to test)
    // example_convenience_methods().await?;
    // example_account_management().await?;

    println!("✅ Examples completed!");
    println!();
    println!("💡 Configuration Tips:");
    println!("   - Edit the constants at the top to enable/disable tests");
    println!("   - Set ENABLE_KEYRING_STORAGE = true to test credential storage");
    println!("   - Set TEST_* flags = true and add real credentials to test providers");
    println!("   - Set TEST_LOGOUT_EXAMPLES = true to test logout functionality");
    println!("   - Set TEST_TOKEN_REFRESH = true to test token refresh with real accounts");
    println!("   - Keyring storage works automatically when you login with real accounts!");
    Ok(())
}
