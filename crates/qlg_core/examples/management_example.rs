//! Account Management Examples
//!
//! This example demonstrates core account management features like:
//! - Account logout functionality
//! - Token refresh and validation
//! - Keyring storage management
//! - Cross-provider account operations

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

/// Configuration
const TEST_LOGOUT_EXAMPLES: bool = true;
const TEST_TOKEN_REFRESH: bool = true;
const TEST_KEYRING_STORAGE: bool = true;

/// Example: Account logout functionality
pub async fn example_logout() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🚪 Account Logout Example");
    println!("==========================");
    println!();

    if !TEST_LOGOUT_EXAMPLES {
        println!("⚠️  Logout testing disabled");
        println!("💡 Set TEST_LOGOUT_EXAMPLES = true to test logout functionality");
        return Ok(());
    }

    // First, let's see what accounts are available
    let accounts = manager.get_accounts().await?;

    if accounts.is_empty() {
        println!("📭 No accounts found in keyring to logout");
        println!("💡 Creating test offline account for demonstration...");
        
        // Create a test account for logout demo
        let test_result = manager.quick_offline_login("LogoutTestUser").await?;
        if let AuthResult::Success(account) = test_result {
            println!("   ✅ Created test account: {}", account.display_username());
        }
    } else {
        println!("📬 Found {} accounts before logout:", accounts.len());
        for account in &accounts {
            println!("   - {} ({})", account.display_username(), account.provider);
        }
    }

    println!();

    // Test creating and then logging out a test account
    println!("🔧 Testing logout with temporary account...");
    let test_result = manager.quick_offline_login("TempLogoutUser").await?;
    let _test_account = match test_result {
        AuthResult::Success(account) => {
            println!("   ✅ Created: {} ({})", account.display_username(), account.provider);
            account
        }
        _ => return Err("Failed to create test account".into()),
    };

    // Now logout the test account
    println!("   🚪 Logging out test account...");
    manager.logout("TempLogoutUser", AccountProvider::Offline).await?;
    println!("   ✅ Test account logged out successfully");

    // Verify it's removed
    let accounts_after = manager.get_accounts().await?;
    let still_exists = accounts_after
        .iter()
        .any(|acc| acc.username == "TempLogoutUser");
    
    if !still_exists {
        println!("   ✅ Account successfully removed from storage");
    } else {
        println!("   ⚠️  Account still found in storage");
    }

    println!();

    // Show logout examples for different providers
    demonstration_logout_examples().await?;

    Ok(())
}

/// Demonstrates the correct way to logout accounts with different providers
async fn demonstration_logout_examples() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AccountManager::new();

    // Get current accounts to show real examples
    let accounts = manager.get_accounts().await?;

    println!("📖 Logout Examples by Provider:");
    println!();

    // Show examples for each provider type
    println!("🔵 **ElyBy Accounts**:");
    println!("   manager.logout(\"username\", AccountProvider::ElyBy).await?;");
    if let Some(elyby_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::ElyBy))
    {
        println!("   // Real example: manager.logout(\"{}\", AccountProvider::ElyBy).await?;", elyby_account.username);
    }
    println!();

    println!("🟢 **Microsoft Accounts**:");
    println!("   manager.logout(\"email@example.com\", AccountProvider::Microsoft).await?;");
    if let Some(ms_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::Microsoft))
    {
        println!("   // Real example: manager.logout(\"{}\", AccountProvider::Microsoft).await?;", ms_account.username);
    } else {
        println!("   // Example: manager.logout(\"player@outlook.com\", AccountProvider::Microsoft).await?;");
    }
    println!();

    println!("🟡 **LittleSkin Accounts**:");
    println!("   manager.logout(\"username\", AccountProvider::LittleSkin).await?;");
    if let Some(ls_account) = accounts
        .iter()
        .find(|a| matches!(a.provider, AccountProvider::LittleSkin))
    {
        println!("   // Real example: manager.logout(\"{}\", AccountProvider::LittleSkin).await?;", ls_account.username);
    }
    println!();

    println!("⚫ **Offline Accounts**:");
    println!("   manager.logout(\"TestPlayer\", AccountProvider::Offline).await?;");
    println!("   // Note: Offline accounts don't store tokens, logout removes from memory");
    println!();

    println!("⚠️  **Important Notes**:");
    println!("   - This removes stored tokens from keyring!");
    println!("   - Users will need to login again after logout!");
    println!("   - Microsoft accounts need OAuth flow again");
    println!("   - ElyBy/LittleSkin accounts need credentials again");

    Ok(())
}

/// Example: Token refresh functionality
pub async fn example_token_refresh() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🔄 Token Refresh Example");
    println!("=========================");
    println!();

    if !TEST_TOKEN_REFRESH {
        println!("⚠️  Token refresh testing disabled");
        println!("💡 Set TEST_TOKEN_REFRESH = true to test token refresh");
        return Ok(());
    }

    // Get existing accounts
    let accounts = manager.get_accounts().await?;

    if accounts.is_empty() {
        println!("📭 No accounts with tokens found for refresh testing");
        println!("💡 This example works best with real logged-in accounts");

        // Demonstrate with offline account (which doesn't need refresh)
        println!();
        println!("🔧 Creating offline account to demonstrate API...");
        let offline_result = manager.quick_offline_login("RefreshTestUser").await?;
        let offline_account = match offline_result {
            AuthResult::Success(account) => account,
            _ => return Err("Failed to create offline account".into()),
        };

        println!("🔍 Checking if token needs refresh...");
        let refreshed = manager.ensure_valid_token(&offline_account).await?;

        if refreshed.needs_refresh {
            println!("   ⚠️  Token needs refresh");
        } else {
            println!("   ✅ Token is valid (offline accounts don't need refresh)");
        }

        // Clean up
        manager.logout("RefreshTestUser", AccountProvider::Offline).await?;
        return Ok(());
    }

    println!("📋 Testing token refresh for {} accounts:", accounts.len());
    println!();

    for account in accounts {
        println!("🔍 Testing: {} ({})", account.display_username(), account.provider);

        // Check current token status
        if account.access_token.is_some() {
            println!("   🔑 Has stored token");
        } else {
            println!("   🔓 No token stored");
        }

        if account.needs_refresh {
            println!("   ⚠️  Token marked as needing refresh");
        } else {
            println!("   ✅ Token appears valid");
        }

        // Use ensure_valid_token to refresh if needed
        println!("   🔄 Ensuring token is valid...");
        match manager.ensure_valid_token(&account).await {
            Ok(refreshed_account) => {
                if refreshed_account.access_token.is_some() {
                    println!("   ✅ Token is valid/refreshed");

                    // Show token info (first few chars only for security)
                    if let Some(token) = &refreshed_account.access_token {
                        let preview = if token.len() > 8 {
                            format!("{}...", &token[..8])
                        } else {
                            "***".to_string()
                        };
                        println!("   🔑 Token preview: {}", preview);
                    }
                } else {
                    println!("   ❌ No valid token available");
                }
            }
            Err(e) => {
                println!("   ❌ Token refresh failed: {}", e);
                println!("   💡 This might mean the refresh token is expired");
                println!("   💡 User may need to login again");
            }
        }
        println!();
    }

    println!("💡 Token Refresh Tips:");
    println!("   - Microsoft accounts: Refresh tokens → new access tokens");
    println!("   - ElyBy/LittleSkin: Access tokens used directly");
    println!("   - Offline accounts: No tokens needed");
    println!("   - Failed refresh usually means re-login required");

    Ok(())
}

/// Example: Keyring storage functionality
pub async fn example_keyring_storage() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🔐 Keyring Storage Example");
    println!("===========================");
    println!();

    if !TEST_KEYRING_STORAGE {
        println!("⚠️  Keyring storage testing disabled");
        println!("💡 Set TEST_KEYRING_STORAGE = true to test keyring functionality");
        return Ok(());
    }

    println!("💡 Keyring stores different token types:");
    println!("   - ElyBy & LittleSkin: Access tokens");
    println!("   - Microsoft: Refresh tokens");
    println!("   - Offline: No tokens (UUID-only)");
    println!();

    // Check current stored accounts
    println!("📋 Checking stored accounts in keyring...");
    let accounts = manager.get_accounts().await?;
    
    if accounts.is_empty() {
        println!("   📭 No accounts found in keyring");
    } else {
        println!("   📬 Found {} stored accounts:", accounts.len());
        for (i, account) in accounts.iter().enumerate() {
            println!("      {}. {} ({}) - UUID: {}", 
                i + 1, 
                account.display_username(), 
                account.provider, 
                account.uuid
            );

            if let Some(_token) = &account.access_token {
                println!("         🔑 Has stored token");
            } else {
                println!("         🔓 No stored token");
            }
        }
    }

    // Test keyring operations with a temporary account
    println!();
    println!("🔧 Testing keyring operations...");
    
    // Create test account
    println!("1. Creating test offline account...");
    let result = manager.quick_offline_login("KeyringTestUser").await?;
    
    match result {
        AuthResult::Success(account) => {
            println!("   ✅ Created: {}", account.display_username());

            // Test retrieval
            println!("2. Testing account retrieval...");
            match manager.get_account("KeyringTestUser", AccountProvider::Offline).await {
                Ok(Some(found_account)) => {
                    println!("   ✅ Found account: {}", found_account.display_username());
                }
                Ok(None) => {
                    println!("   ℹ️  Account not found (expected for offline accounts)");
                }
                Err(e) => {
                    println!("   ❌ Error retrieving account: {}", e);
                }
            }

            // Test removal
            println!("3. Testing account removal...");
            match manager.logout("KeyringTestUser", AccountProvider::Offline).await {
                Ok(()) => {
                    println!("   ✅ Account removed successfully");
                }
                Err(e) => {
                    println!("   ❌ Failed to remove account: {}", e);
                }
            }

            // Verify removal
            println!("4. Verifying removal...");
            let accounts_after = manager.get_accounts().await?;
            let still_exists = accounts_after
                .iter()
                .any(|acc| acc.username == "KeyringTestUser");
            
            if !still_exists {
                println!("   ✅ Account successfully removed from keyring");
            } else {
                println!("   ⚠️  Account still found in keyring");
            }
        }
        _ => {
            println!("   ❌ Failed to create test account");
        }
    }

    println!();
    println!("🔐 Keyring Security Notes:");
    println!("   - Tokens are encrypted by the OS keyring");
    println!("   - Different apps can't access each other's tokens");
    println!("   - Tokens persist across launcher restarts");
    println!("   - Manual keyring cleanup may be needed on uninstall");

    Ok(())
}

/// Example: Show all stored accounts
pub async fn example_show_all_accounts() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AccountManager::new();

    println!();
    println!("📋 All Stored Accounts");
    println!("=======================");
    println!();

    match manager.get_accounts().await {
        Ok(accounts) => {
            if accounts.is_empty() {
                println!("📭 No accounts found in storage");
                println!("💡 Login with any provider to see accounts here");
            } else {
                println!("📬 Found {} total accounts:", accounts.len());
                
                // Group by provider
                let mut by_provider: std::collections::HashMap<AccountProvider, Vec<_>> = 
                    std::collections::HashMap::new();
                
                for account in accounts {
                    by_provider.entry(account.provider).or_insert_with(Vec::new).push(account);
                }

                for (provider, provider_accounts) in by_provider {
                    println!();
                    println!("   {} {} account(s):", provider_accounts.len(), provider);
                    for (i, account) in provider_accounts.iter().enumerate() {
                        println!("      {}. {} (UUID: {})", 
                            i + 1, 
                            account.display_username(), 
                            account.uuid
                        );
                        
                        if account.access_token.is_some() {
                            println!("         🔑 Has token");
                        } else {
                            println!("         🔓 No token");
                        }

                        if account.needs_refresh {
                            println!("         ⚠️  Needs refresh");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to access account storage: {}", e);
            println!("💡 This might be normal if keyring is not available");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - Account Management Examples");
    println!("======================================");
    println!();

    // Show current accounts first
    example_show_all_accounts().await?;

    // Run management examples
    if TEST_KEYRING_STORAGE {
        example_keyring_storage().await?;
    }

    if TEST_LOGOUT_EXAMPLES {
        example_logout().await?;
    }

    if TEST_TOKEN_REFRESH {
        example_token_refresh().await?;
    }

    println!();
    println!("✅ Account management examples completed!");
    println!();
    println!("💡 Configuration:");
    println!("   - Set TEST_LOGOUT_EXAMPLES = true to test logout");
    println!("   - Set TEST_TOKEN_REFRESH = true to test token refresh");
    println!("   - Set TEST_KEYRING_STORAGE = true to test keyring operations");
    println!();
    println!("💡 Related Examples:");
    println!("   - Run provider-specific examples for login functionality");
    println!("   - cargo run --example elyby_example");
    println!("   - cargo run --example microsoft_example");
    println!("   - cargo run --example littleskin_example");
    println!("   - cargo run --example offline_example");

    Ok(())
}
