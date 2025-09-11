//! # QLG Core Usage Examples
//! 
//! This file demonstrates how to use the QLG Core account management system.

use qlg_core::{
    AccountManager, AccountProvider, LoginCredentials, AuthResult, AccountManagerTrait
};

/// Example: Simple login with ElyBy
pub async fn example_elyby_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();
    
    let credentials = LoginCredentials {
        username: "your_username".to_string(),
        password: "your_password".to_string(),
        totp_code: None,
    };
    
    match manager.login(AccountProvider::ElyBy, &credentials).await? {
        AuthResult::Success(account) => {
            println!("Login successful! Welcome, {}", account.display_name);
            println!("UUID: {}", account.uuid);
        }
        AuthResult::RequiresTwoFactor => {
            println!("Two-factor authentication required");
            // You can retry with TOTP code
        }
        AuthResult::Failed(error) => {
            println!("Login failed: {}", error);
        }
    }
    
    Ok(())
}

/// Example: Microsoft OAuth login
pub async fn example_microsoft_oauth() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();
    
    // Start OAuth flow
    let oauth_flow = manager.start_oauth_login(AccountProvider::Microsoft).await?;
    
    println!("Please visit: {}", oauth_flow.verification_uri);
    println!("Enter code: {}", oauth_flow.user_code);
    println!("Waiting for authorization...");
    
    // In a real app, you'd wait for user to complete the flow
    // For now, let's just show how to complete it
    match manager.complete_oauth_login(AccountProvider::Microsoft, &oauth_flow.device_code).await? {
        AuthResult::Success(account) => {
            println!("OAuth login successful! Welcome, {}", account.display_name);
        }
        AuthResult::Failed(error) => {
            println!("OAuth login failed: {}", error);
        }
        _ => unreachable!("Microsoft OAuth doesn't use 2FA"),
    }
    
    Ok(())
}

/// Example: Quick login with convenience method
pub async fn example_quick_login() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();
    
    // Simple username/password login
    match manager.quick_login(AccountProvider::ElyBy, "username", "password").await? {
        AuthResult::Success(account) => {
            println!("Quick login successful: {}", account.display_username());
        }
        AuthResult::RequiresTwoFactor => {
            // Retry with 2FA
            match manager.quick_login_with_2fa(AccountProvider::ElyBy, "username", "password", "123456").await? {
                AuthResult::Success(account) => {
                    println!("2FA login successful: {}", account.display_username());
                }
                AuthResult::Failed(error) => {
                    println!("2FA login failed: {}", error);
                }
                _ => {}
            }
        }
        AuthResult::Failed(error) => {
            println!("Quick login failed: {}", error);
        }
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
        let (creds, oauth) = manager.provider_capabilities(provider)?;
        println!("{}: credentials={}, oauth={}", provider, creds, oauth);
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
    // These are just examples - don't run them all at once in a real app
    println!("QLG Core Account Management Examples");
    println!("=====================================");
    
    // Uncomment the examples you want to try:
    // example_elyby_login().await?;
    // example_microsoft_oauth().await?;
    // example_quick_login().await?;
    // example_account_management().await?;
    // example_logout().await?;
    
    Ok(())
}
