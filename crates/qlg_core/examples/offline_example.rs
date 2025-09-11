//! Offline Account Provider Example
//!
//! This example demonstrates how to use offline (cracked) authentication with QLG Core.
//! Offline accounts don't require internet connectivity or real credentials.

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

/// Example: Offline (username-only) login
pub async fn example_offline_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("⚫ Offline Authentication Example");
    println!("=================================");
    println!();

    println!("💡 Offline accounts are perfect for:");
    println!("   - Testing and development");
    println!("   - Playing on cracked servers");
    println!("   - Local/LAN gameplay");
    println!("   - When internet is unavailable");
    println!();

    // Simple offline login with just username
    println!("🔄 Creating offline account with username 'TestPlayer'...");
    match manager.quick_offline_login("TestPlayer").await? {
        AuthResult::Success(account) => {
            println!("✅ Offline login successful!");
            println!("   Display name: {}", account.display_name);
            println!("   Username: {}", account.username);
            println!("   UUID: {}", account.uuid);
            println!("   Provider: {}", account.provider);
            println!("   Access token: {:?}", account.access_token);
            
            // Offline accounts don't have access tokens
            if account.access_token.is_none() {
                println!("   💡 Offline accounts don't need access tokens");
            }
        }
        AuthResult::Failed(error) => {
            println!("❌ Offline login failed: {}", error);
        }
        _ => unreachable!("Offline login doesn't use 2FA"),
    }

    println!();

    // You can also use the longer form
    println!("🔄 Creating another offline account with 'AnotherPlayer'...");
    match manager
        .login_username_only(AccountProvider::Offline, "AnotherPlayer")
        .await?
    {
        AuthResult::Success(account) => {
            println!("✅ Username-only login successful!");
            println!("   Account: {}", account.display_username());
            println!("   UUID: {}", account.uuid);
        }
        _ => {}
    }

    Ok(())
}

/// Example: Offline account validation and management
pub async fn example_offline_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🔍 Offline Account Validation Example");
    println!("======================================");
    println!();

    // Test username validation
    let test_usernames = vec![
        ("ValidUser123", true),
        ("User_Name", true),
        ("ab", false), // Too short
        ("", false),   // Empty
        ("ThisUsernameIsWayTooLongToBeValid1234567890", false), // Too long
        ("Valid123", true),
        ("Test_Player", true),
    ];

    println!("Testing username validation:");
    for (username, should_be_valid) in test_usernames {
        print!("   '{}' -> ", username);
        
        match manager.quick_offline_login(username).await {
            Ok(AuthResult::Success(_)) => {
                println!("✅ Valid");
                if !should_be_valid {
                    println!("      ⚠️  Expected to be invalid!");
                }
                
                // Clean up
                let _ = manager.logout(username, AccountProvider::Offline).await;
            }
            Ok(AuthResult::Failed(error)) => {
                println!("❌ Invalid: {}", error);
                if should_be_valid {
                    println!("      ⚠️  Expected to be valid!");
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
            _ => unreachable!("Offline doesn't use 2FA"),
        }
    }

    Ok(())
}

/// Example: Working with multiple offline accounts
pub async fn example_offline_multiple_accounts() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("👥 Multiple Offline Accounts Example");
    println!("=====================================");
    println!();

    // Create several offline accounts
    let usernames = vec!["Player1", "TestUser", "GameDev", "LocalPlayer"];
    
    println!("Creating {} offline accounts...", usernames.len());
    for username in &usernames {
        match manager.quick_offline_login(username).await? {
            AuthResult::Success(account) => {
                println!("   ✅ Created: {} (UUID: {})", account.display_username(), account.uuid);
            }
            AuthResult::Failed(error) => {
                println!("   ❌ Failed to create {}: {}", username, error);
            }
            _ => {}
        }
    }

    println!();

    // Check stored accounts
    println!("📋 Checking stored accounts...");
    let accounts = manager.get_accounts().await?;
    let offline_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::Offline))
        .collect();

    if offline_accounts.is_empty() {
        println!("   📭 No offline accounts found in memory");
    } else {
        println!("   📬 Found {} offline accounts:", offline_accounts.len());
        for (i, account) in offline_accounts.iter().enumerate() {
            println!("      {}. {} (UUID: {})", i + 1, account.display_username(), account.uuid);
            
            // Test "token" validation (offline accounts always pass)
            match manager.ensure_valid_token(account).await {
                Ok(validated) => {
                    if validated.access_token.is_none() {
                        println!("         💡 No token needed (offline account)");
                    } else {
                        println!("         🔑 Has token");
                    }
                }
                Err(e) => {
                    println!("         ❌ Validation error: {}", e);
                }
            }
        }
    }

    println!();

    // Demonstrate cleanup
    println!("🧹 Cleaning up test accounts...");
    for username in &usernames {
        match manager.logout(username, AccountProvider::Offline).await {
            Ok(()) => {
                println!("   ✅ Removed: {}", username);
            }
            Err(e) => {
                println!("   ⚠️  Failed to remove {}: {}", username, e);
            }
        }
    }

    // Verify cleanup
    let remaining_accounts = manager.get_accounts().await?;
    let remaining_offline: Vec<_> = remaining_accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::Offline))
        .collect();
    
    println!("   📋 Remaining offline accounts: {}", remaining_offline.len());

    Ok(())
}

/// Example: Offline account capabilities and limitations
pub async fn example_offline_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AccountManager::new();

    println!();
    println!("📋 Offline Provider Capabilities");
    println!("=================================");
    println!();

    // Show provider capabilities
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::Offline)?;
    println!("Offline Provider Features:");
    println!("   Credentials: {} (not needed)", creds);
    println!("   OAuth: {} (not supported)", oauth);
    println!("   Username-only: {} (main feature)", username_only);
    println!();

    println!("✅ What Offline Accounts Support:");
    println!("   • Username-only authentication");
    println!("   • Instant account creation");
    println!("   • No internet connection required");
    println!("   • Perfect for testing and development");
    println!("   • Works with cracked/offline Minecraft servers");
    println!("   • No token storage needed");
    println!();

    println!("❌ What Offline Accounts Don't Support:");
    println!("   • Access to official Minecraft servers");
    println!("   • Skin/cape downloads from Mojang");
    println!("   • Premium account features");
    println!("   • Token-based authentication");
    println!("   • Account validation with Mojang");
    println!();

    println!("🎯 Best Use Cases:");
    println!("   • Development and testing");
    println!("   • Local multiplayer servers");
    println!("   • Cracked server networks");
    println!("   • Offline single-player mode");
    println!("   • Educational environments");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - Offline Provider Example");
    println!("====================================");
    println!();

    // Run offline examples
    example_offline_login().await?;
    example_offline_validation().await?;
    example_offline_multiple_accounts().await?;
    example_offline_capabilities().await?;

    println!();
    println!("✅ Offline examples completed!");
    println!();
    println!("💡 Key Points:");
    println!("   - Offline accounts require only a username");
    println!("   - No internet connection or tokens needed");
    println!("   - Perfect for development and testing");
    println!("   - Work with cracked servers and local play");
    println!("   - UUIDs are generated automatically");

    Ok(())
}
