//! Microsoft Account Provider Example
//!
//! This example demonstrates how to use Microsoft authentication with QLG Core.
//! Microsoft accounts use OAuth2 device code flow for secure authentication.

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

/// Configuration: Set to true to test OAuth flow
const TEST_OAUTH_FLOW: bool = false;

/// Example: Microsoft OAuth login  
pub async fn example_microsoft_oauth() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!("🟢 Microsoft OAuth Authentication Example");
    println!("==========================================");
    println!();

    if !TEST_OAUTH_FLOW {
        println!("⚠️  OAuth flow testing disabled");
        println!("💡 Set TEST_OAUTH_FLOW = true to test Microsoft OAuth");
        println!();
        return Ok(());
    }

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
                println!("   UUID: {}", account.uuid);
                println!("   Provider: {}", account.provider);
                println!("   Username: {}", account.username);

                if account.access_token.is_some() {
                    println!("   🔑 Refresh token stored in keyring");
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

/// Example: Working with stored Microsoft accounts
pub async fn example_microsoft_account_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🗃️  Microsoft Account Management");
    println!("=================================");
    println!();

    // Check for existing Microsoft accounts
    let accounts = manager.get_accounts().await?;
    let microsoft_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::Microsoft))
        .collect();

    if microsoft_accounts.is_empty() {
        println!("📭 No Microsoft accounts found in keyring");
        println!("💡 Complete OAuth flow first to see account management features");
        return Ok(());
    }

    println!("📬 Found {} Microsoft accounts:", microsoft_accounts.len());

    for (i, account) in microsoft_accounts.iter().enumerate() {
        println!(
            "   {}. {} (UUID: {})",
            i + 1,
            account.display_username(),
            account.uuid
        );

        if account.access_token.is_some() {
            println!("      🔑 Has stored refresh token");
        } else {
            println!("      🔓 No refresh token");
        }

        // Test token refresh if needed
        if account.needs_refresh {
            println!("      🔄 Token needs refresh...");
            match manager.ensure_valid_token(account).await {
                Ok(refreshed) => {
                    if refreshed.access_token.is_some() {
                        println!("      ✅ New access token obtained");
                    } else {
                        println!("      ❌ Failed to get new access token");
                    }
                }
                Err(e) => {
                    println!("      ❌ Token refresh failed: {}", e);
                    println!("      💡 Account may need to re-authorize");
                }
            }
        } else {
            println!("      ✅ Token is valid");
        }
    }

    // Show provider capabilities
    println!();
    println!("📋 Microsoft Provider Capabilities:");
    let (creds, oauth, username_only) =
        manager.provider_capabilities(AccountProvider::Microsoft)?;
    println!("   Credentials: {} (not supported)", creds);
    println!("   OAuth: {} (device code flow)", oauth);
    println!("   Username-only: {} (not supported)", username_only);

    // Demonstrate logout (commented for safety)
    println!();
    println!("🚪 How to logout Microsoft accounts:");
    for account in &microsoft_accounts {
        println!(
            "   manager.logout(\"{}\", AccountProvider::Microsoft).await?;",
            account.username
        );
    }
    println!("   ⚠️  This will remove the stored refresh token!");
    println!("   ⚠️  You'll need to complete OAuth flow again to re-login!");

    Ok(())
}

/// Example: Microsoft account token management
pub async fn example_microsoft_token_refresh() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    println!();
    println!("🔄 Microsoft Token Refresh Example");
    println!("===================================");
    println!();

    let accounts = manager.get_accounts().await?;
    let microsoft_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|acc| matches!(acc.provider, AccountProvider::Microsoft))
        .collect();

    if microsoft_accounts.is_empty() {
        println!("📭 No Microsoft accounts found for token refresh testing");
        return Ok(());
    }

    for account in microsoft_accounts {
        println!(
            "🔍 Testing token refresh for: {}",
            account.display_username()
        );

        // Microsoft accounts use refresh tokens to get new access tokens
        println!("   💡 Microsoft uses refresh tokens to obtain fresh access tokens");

        if account.access_token.is_some() {
            println!("   🔑 Has refresh token stored");
        } else {
            println!("   🔓 No refresh token (account needs re-authorization)");
            continue;
        }

        // Force a token refresh
        println!("   🔄 Requesting fresh access token...");
        match manager.ensure_valid_token(&account).await {
            Ok(refreshed) => {
                if refreshed.access_token.is_some() {
                    println!("   ✅ Fresh access token obtained");

                    // Show token preview (first few chars only)
                    if let Some(token) = &refreshed.access_token {
                        let preview = if token.len() > 12 {
                            format!("{}...", &token[..12])
                        } else {
                            "***".to_string()
                        };
                        println!("   🔑 Token preview: {}", preview);
                    }
                } else {
                    println!("   ❌ Failed to obtain access token");
                }
            }
            Err(e) => {
                println!("   ❌ Token refresh failed: {}", e);
                println!("   💡 The refresh token may be expired");
                println!("   💡 User needs to complete OAuth flow again");
            }
        }

        println!();
    }

    println!("💡 Microsoft Token Management:");
    println!("   - Refresh tokens are long-lived (stored in keyring)");
    println!("   - Access tokens are short-lived (requested as needed)");
    println!("   - If refresh fails, user must re-authorize via OAuth");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - Microsoft Provider Example");
    println!("======================================");
    println!();

    // Show provider info first
    let manager = AccountManager::new();
    let (creds, oauth, username_only) =
        manager.provider_capabilities(AccountProvider::Microsoft)?;
    println!("📋 Microsoft Provider Capabilities:");
    println!("   Credentials: {} (Microsoft uses OAuth only)", creds);
    println!("   OAuth: {} (device code flow)", oauth);
    println!("   Username-only: {} (not supported)", username_only);
    println!();

    // Run Microsoft examples
    example_microsoft_oauth().await?;
    example_microsoft_account_management().await?;
    example_microsoft_token_refresh().await?;

    println!();
    println!("✅ Microsoft examples completed!");
    println!();
    println!("💡 Tips:");
    println!("   - Set TEST_OAUTH_FLOW = true to test OAuth authentication");
    println!("   - Microsoft uses device code flow (browser-based)");
    println!("   - Refresh tokens are stored securely for automatic access token renewal");
    println!("   - If refresh tokens expire, users must re-authorize");

    Ok(())
}
