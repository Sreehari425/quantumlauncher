//! ElyBy Account Provider Example
//!
//! This example demonstrates how to use ElyBy authentication with QLG Core.
//! ElyBy is a popular Minecraft authentication service that supports credential-based login.

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

/// Configuration: Set to true and add real credentials to test
const TEST_WITH_REAL_CREDENTIALS: bool = false;

/// Example: ElyBy login with username and password
pub async fn example_elyby_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Replace with your actual ElyBy credentials
    let username = "your_email_or_username";
    let password = "your_password";

    println!("🔵 ElyBy Authentication Example");
    println!("================================");
    println!();

    if !TEST_WITH_REAL_CREDENTIALS {
        println!("⚠️  Testing with dummy credentials (will fail)");
        println!("💡 Set TEST_WITH_REAL_CREDENTIALS = true and add real credentials to test");
        println!();
    }

    println!("Attempting ElyBy login for: {}", username);

    match manager
        .quick_login(AccountProvider::ElyBy, username, password)
        .await?
    {
        AuthResult::Success(account) => {
            println!(
                "✅ ElyBy login successful! Welcome, {}",
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
            println!("🔐 Two-factor authentication required");
            println!("💡 You can retry with TOTP code:");
            println!("   manager.quick_login_with_2fa(AccountProvider::ElyBy, username, password, \"123456\").await?;");
        }
        AuthResult::Failed(error) => {
            println!("❌ ElyBy login failed: {}", error);

            if !TEST_WITH_REAL_CREDENTIALS {
                println!("💡 This is expected with dummy credentials");
            } else {
                println!("💡 Check your credentials and try again");
                println!("💡 Make sure your ElyBy account is active");
            }
        }
    }

    // Show provider capabilities
    println!();
    println!("📋 ElyBy Provider Capabilities:");
    let (creds, oauth, username_only) = manager.provider_capabilities(AccountProvider::ElyBy)?;
    println!("   Credentials: {} (username/password login)", creds);
    println!("   OAuth: {} (browser-based login)", oauth);
    println!("   Username-only: {} (offline mode)", username_only);

    // Test credential validation
    println!();
    println!("🔍 Testing credential validation...");

    // Note: Direct provider access is not available in the public API
    // Instead, we'll attempt login to validate credentials
    println!("⚠️  Note: Credential validation requires actual login attempt");
    println!("         This is a limitation of the current public API");

    Ok(())
}

/// Example: Working with stored ElyBy accounts
pub async fn example_elyby_account_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🗃️  ElyBy Account Management");
    println!("============================");
    println!();

    // Check for existing ElyBy accounts
    let accounts = manager.get_accounts().await?;
    let elyby_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::ElyBy))
        .collect();

    if elyby_accounts.is_empty() {
        println!("📭 No ElyBy accounts found in keyring");
        println!("💡 Login with real credentials first to see account management features");
        return Ok(());
    }

    println!("📬 Found {} ElyBy accounts:", elyby_accounts.len());

    for (i, account) in elyby_accounts.iter().enumerate() {
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

    // Demonstrate logout (commented for safety)
    println!();
    println!("🚪 How to logout ElyBy accounts:");
    for account in &elyby_accounts {
        println!(
            "   manager.logout(\"{}\", AccountProvider::ElyBy).await?;",
            account.username
        );
    }
    println!("   ⚠️  This will remove the stored access token!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - ElyBy Provider Example");
    println!("==================================");
    println!();

    // Run ElyBy examples
    example_elyby_login().await?;
    example_elyby_account_management().await?;

    println!();
    println!("✅ ElyBy examples completed!");
    println!();
    println!("💡 Tips:");
    println!("   - Set TEST_WITH_REAL_CREDENTIALS = true to test with real accounts");
    println!("   - ElyBy uses access tokens stored securely in the system keyring");
    println!("   - Tokens are automatically validated and refreshed when needed");
    println!("   - Use manager.logout() to remove stored tokens");

    Ok(())
}
