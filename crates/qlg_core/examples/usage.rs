//! # QLG Core Usage Examples
//!
//! This file demonstrates how to use the QLG Core account management system.

//! Tip: dont commit you actual username and password (happend to me)

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider, AuthResult};

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
     HOW TO TEST EACH PROVIDER:

    1.  ElyBy (Credentials only):
       - Uncomment example_elyby_login()
       - Replace "your_elyby_username" and "your_elyby_password" with real credentials

    2.  Microsoft (OAuth only):
       - Uncomment example_microsoft_oauth()
       - Follow the OAuth flow: visit URL, enter code, get authorization code
       - Replace "AUTHORIZATION_CODE_HERE" with the actual code

    3.  LittleSkin (Both methods):
       - For credentials: uncomment example_littleskin_credentials()
       - Replace email/username and password with real LittleSkin credentials
       - For OAuth: uncomment example_littleskin_oauth()
       - Follow OAuth flow similar to Microsoft

    4.  Offline (Username-only):
       - Already enabled below - works with any valid username
       - No credentials required, generates deterministic UUID
    */

    // Show all available provider capabilities first
    let manager = AccountManager::new();
    println!(" Supported Providers and Their Capabilities:");
    for provider in manager.supported_providers() {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        println!(
            "  {} - Credentials: {}, OAuth: {}, Username-Only: {}",
            provider, creds, oauth, username_only
        );
    }
    println!();

    // Uncomment the examples you want to try:

    //  ElyBy (Credentials only)
    // println!(" Testing ElyBy Login...");
    // example_elyby_login().await?;
    // println!();

    //  Microsoft (OAuth only)
    // println!(" Testing Microsoft OAuth...");
    // example_microsoft_oauth().await?;
    // println!();

    //  LittleSkin (Both credentials and OAuth)
    // cred flow
    // println!(" Testing LittleSkin Credential Login...");
    // example_littleskin_credentials().await?;
    // println!();
    // Device code flow below
    // println!(" Testing LittleSkin OAuth...");
    // example_littleskin_oauth().await?;
    // println!();

    //  Offline (Username-only) - No credentials needed!
    println!(" Testing Offline Login...");
    example_offline_login().await?;
    println!();

    //  Other examples
    // example_convenience_methods().await?;
    // example_account_management().await?;
    // example_logout().await?;

    println!(" Examples completed!");
    println!(" Tip: Uncomment and configure the examples above to test with real credentials!");
    Ok(())
}
