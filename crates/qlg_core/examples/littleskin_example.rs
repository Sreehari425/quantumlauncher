//! LittleSkin Account Provider Example
//!
//! This example demonstrates how to use LittleSkin authentication with QLG Core.
//! LittleSkin supports both credential-based login and OAuth flow.

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

/// Configuration: Set to true and add real credentials to test
const TEST_WITH_REAL_CREDENTIALS: bool = false;
const TEST_OAUTH_FLOW: bool = true;

/// Example: LittleSkin credential login (email/username + password)
pub async fn example_littleskin_credentials() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Replace with your actual LittleSkin credentials
    let email_or_username = "your_email_or_username";
    let password = "your_password";

    println!("🟡 LittleSkin Credential Authentication Example");
    println!("===============================================");
    println!();

    if !TEST_WITH_REAL_CREDENTIALS {
        println!("⚠️  Testing with dummy credentials (will fail)");
        println!("💡 Set TEST_WITH_REAL_CREDENTIALS = true and add real credentials to test");
        println!();
    }

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
                "✅ LittleSkin credential login successful! Welcome, {}",
                account.display_name
            );
            println!("   UUID: {}", account.uuid);
            println!("   Provider: {}", account.provider);
            println!("   Username: {}", account.username);

            if account.access_token.is_some() {
                println!("   🔑 Access token stored in keyring");
            }

            // Test token validation
            println!();
            println!("🔄 Testing token validation...");
            let validated_account = manager.ensure_valid_token(&account).await?;
            if validated_account.access_token.is_some() {
                println!("✅ Token is valid");
            } else {
                println!("⚠️  Token needs refresh");
            }
        }
        AuthResult::RequiresTwoFactor => {
            println!("🔐 Two-factor authentication required for LittleSkin");
            println!("💡 You can retry with TOTP code:");
            println!("   manager.quick_login_with_2fa(AccountProvider::LittleSkin, email_or_username, password, \"123456\").await?;");
        }
        AuthResult::Failed(error) => {
            println!("❌ LittleSkin credential login failed: {}", error);

            if !TEST_WITH_REAL_CREDENTIALS {
                println!("💡 This is expected with dummy credentials");
            } else {
                println!("💡 Check your credentials and try again");
                println!("💡 Make sure your LittleSkin account is active");
            }
        }
    }

    Ok(())
}

/// Example: LittleSkin OAuth login
pub async fn example_littleskin_oauth() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🟡 LittleSkin OAuth Authentication Example");
    println!("===========================================");
    println!();

    if !TEST_OAUTH_FLOW {
        println!("⚠️  OAuth flow testing disabled");
        println!("💡 Set TEST_OAUTH_FLOW = true to test LittleSkin OAuth");
        println!();
        return Ok(());
    }

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
                println!("   UUID: {}", account.uuid);
                println!("   Provider: {}", account.provider);
                println!("   Username: {}", account.username);

                if account.access_token.is_some() {
                    println!("   🔑 Access token stored in keyring");
                }

                // Test token validation
                println!();
                println!("🔄 Testing token validation...");
                let validated_account = manager.ensure_valid_token(&account).await?;
                if validated_account.access_token.is_some() {
                    println!("✅ Token is valid");
                } else {
                    println!("⚠️  Token needs refresh");
                }
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

/// Example: Working with stored LittleSkin accounts
pub async fn example_littleskin_account_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🗃️  LittleSkin Account Management");
    println!("=================================");
    println!();

    // Check for existing LittleSkin accounts
    let accounts = manager.get_accounts().await?;
    let littleskin_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::LittleSkin))
        .collect();

    if littleskin_accounts.is_empty() {
        println!("📭 No LittleSkin accounts found in keyring");
        println!("💡 Login with real credentials first to see account management features");
        return Ok(());
    }

    println!(
        "📬 Found {} LittleSkin accounts:",
        littleskin_accounts.len()
    );

    for (i, account) in littleskin_accounts.iter().enumerate() {
        println!(
            "   {}. {} (UUID: {})",
            i + 1,
            account.display_username(),
            account.uuid
        );

        if account.access_token.is_some() {
            println!("      🔑 Has stored access token");
        } else {
            println!("      🔓 No access token");
        }

        // Test token refresh if needed
        if account.needs_refresh {
            println!("      🔄 Token needs refresh...");
            match manager.ensure_valid_token(account).await {
                Ok(refreshed) => {
                    if refreshed.access_token.is_some() {
                        println!("      ✅ Token refreshed successfully");
                    } else {
                        println!("      ❌ Failed to refresh token");
                    }
                }
                Err(e) => {
                    println!("      ❌ Token refresh failed: {}", e);
                    println!("      💡 Account may need to login again");
                }
            }
        } else {
            println!("      ✅ Token is valid");
        }
    }

    // Show provider capabilities
    println!();
    println!("📋 LittleSkin Provider Capabilities:");
    let (creds, oauth, username_only) =
        manager.provider_capabilities(AccountProvider::LittleSkin)?;
    println!("   Credentials: {} (username/password login)", creds);
    println!("   OAuth: {} (device code flow)", oauth);
    println!("   Username-only: {} (not supported)", username_only);

    // Test credential validation
    if TEST_WITH_REAL_CREDENTIALS {
        println!();
        println!("🔍 Testing credential validation...");

        // Note: Direct provider access is not available in the public API
        // Instead, we'll attempt login to validate credentials
        println!("⚠️  Note: Credential validation requires actual login attempt");
        println!("         This is a limitation of the current public API");
    }

    // Demonstrate logout (commented for safety)
    println!();
    println!("🚪 How to logout LittleSkin accounts:");
    for account in &littleskin_accounts {
        println!(
            "   manager.logout(\"{}\", AccountProvider::LittleSkin).await?;",
            account.username
        );
    }
    println!("   ⚠️  This will remove the stored access token!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - LittleSkin Provider Example");
    println!("=======================================");
    println!();

    // Show provider info first
    let manager = AccountManager::new();
    let (creds, oauth, username_only) =
        manager.provider_capabilities(AccountProvider::LittleSkin)?;
    println!("📋 LittleSkin Provider Capabilities:");
    println!("   Credentials: {} (username/password)", creds);
    println!("   OAuth: {} (device code flow)", oauth);
    println!("   Username-only: {} (not supported)", username_only);
    println!();

    // Run LittleSkin examples
    if TEST_WITH_REAL_CREDENTIALS {
        example_littleskin_credentials().await?;
    }
    if TEST_OAUTH_FLOW {
        example_littleskin_oauth().await?;
    }

    example_littleskin_account_management().await?;

    println!();
    println!("✅ LittleSkin examples completed!");
    println!();
    println!("💡 Tips:");
    println!("   - Set TEST_WITH_REAL_CREDENTIALS = true to test credential login");
    println!("   - Set TEST_OAUTH_FLOW = true to test OAuth authentication");
    println!("   - LittleSkin supports both credential and OAuth authentication");
    println!("   - Access tokens are stored securely in the system keyring");
    println!("   - Use manager.logout() to remove stored tokens");

    Ok(())
}
