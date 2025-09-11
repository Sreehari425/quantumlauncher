//! QLG Core Overview Example
//!
//! This example provides a comprehensive overview of all QLG Core features
//! and demonstrates how to work with multiple authentication providers.

use qlg_core::{AccountManager, AccountManagerTrait, AccountProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core - Comprehensive Overview");
    println!("==================================");
    println!();

    let manager = AccountManager::new();

    // Show all supported providers and their capabilities
    println!("📋 Supported Authentication Providers:");
    println!();

    for provider in manager.supported_providers() {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        
        match provider {
            AccountProvider::Microsoft => {
                println!("🟢 **Microsoft**");
                println!("   • Authentication: OAuth 2.0 (device code flow)");
                println!("   • Token Type: Refresh tokens → Access tokens");
                println!("   • Use Case: Premium Minecraft accounts");
                println!("   • Internet: Required");
                println!("   • Supports: Official Minecraft servers, realms, marketplace");
                println!("   • Credentials: {} | OAuth: {} | Username-only: {}", creds, oauth, username_only);
                println!("   • Example: cargo run --example microsoft_example");
            }
            AccountProvider::ElyBy => {
                println!("🔵 **ElyBy**");
                println!("   • Authentication: Username/Password");
                println!("   • Token Type: Access tokens");
                println!("   • Use Case: Alternative Minecraft authentication");
                println!("   • Internet: Required for login");
                println!("   • Supports: ElyBy-compatible servers, skins");
                println!("   • Credentials: {} | OAuth: {} | Username-only: {}", creds, oauth, username_only);
                println!("   • Example: cargo run --example elyby_example");
            }
            AccountProvider::LittleSkin => {
                println!("🟡 **LittleSkin**");
                println!("   • Authentication: Username/Password OR OAuth");
                println!("   • Token Type: Access tokens");
                println!("   • Use Case: Chinese Minecraft community");
                println!("   • Internet: Required for login");
                println!("   • Supports: LittleSkin servers, custom skins");
                println!("   • Credentials: {} | OAuth: {} | Username-only: {}", creds, oauth, username_only);
                println!("   • Example: cargo run --example littleskin_example");
            }
            AccountProvider::Offline => {
                println!("⚫ **Offline**");
                println!("   • Authentication: Username only");
                println!("   • Token Type: None (UUID-based)");
                println!("   • Use Case: Development, testing, cracked servers");
                println!("   • Internet: Not required");
                println!("   • Supports: Offline servers, local development");
                println!("   • Credentials: {} | OAuth: {} | Username-only: {}", creds, oauth, username_only);
                println!("   • Example: cargo run --example offline_example");
            }
        }
        println!();
    }

    // Show current accounts
    println!("🗃️  Currently Stored Accounts:");
    match manager.get_accounts().await {
        Ok(accounts) => {
            if accounts.is_empty() {
                println!("   📭 No accounts found");
                println!("   💡 Run provider-specific examples to create accounts");
            } else {
                println!("   📬 Found {} accounts:", accounts.len());
                for (i, account) in accounts.iter().enumerate() {
                    let token_status = if account.access_token.is_some() {
                        if account.needs_refresh { "🔄 needs refresh" } else { "🔑 valid" }
                    } else {
                        "🔓 no token"
                    };
                    
                    println!("      {}. {} ({}) - {}", 
                        i + 1, 
                        account.display_username(), 
                        account.provider,
                        token_status
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to access accounts: {}", e);
        }
    }

    println!();
    println!("🎯 Quick Start Guide:");
    println!("======================");
    println!();
    
    println!("1. **For Testing & Development:**");
    println!("   cargo run --example offline_example");
    println!("   • No credentials needed");
    println!("   • Instant account creation");
    println!("   • Perfect for development");
    println!();

    println!("2. **For Premium Minecraft:**");
    println!("   cargo run --example microsoft_example");
    println!("   • Set TEST_OAUTH_FLOW = true");
    println!("   • Complete browser OAuth flow");
    println!("   • Access to official servers");
    println!();

    println!("3. **For Alternative Auth:**");
    println!("   cargo run --example elyby_example");
    println!("   cargo run --example littleskin_example");
    println!("   • Set TEST_WITH_REAL_CREDENTIALS = true");
    println!("   • Add your credentials to test");
    println!("   • Access to alternative servers");
    println!();

    println!("4. **For Account Management:**");
    println!("   cargo run --example management_example");
    println!("   • Test logout functionality");
    println!("   • Token refresh operations");
    println!("   • Keyring storage management");
    println!();

    println!("🔧 Example Configuration:");
    println!("==========================");
    println!();
    println!("Each example has configuration constants at the top:");
    println!("• TEST_WITH_REAL_CREDENTIALS = true  // Enable real auth testing");
    println!("• TEST_OAUTH_FLOW = true             // Enable OAuth testing");
    println!("• TEST_LOGOUT_EXAMPLES = true        // Enable logout testing");
    println!("• TEST_TOKEN_REFRESH = true          // Enable token testing");
    println!();

    println!("⚠️  Security Notes:");
    println!("===================");
    println!();
    println!("• Never commit real credentials to version control");
    println!("• Tokens are stored securely in OS keyring");
    println!("• Logout removes stored tokens permanently");
    println!("• Different providers have different token lifetimes");
    println!();

    println!("🚀 Integration Examples:");
    println!("=========================");
    println!();
    println!("```rust");
    println!("use qlg_core::{{AccountManager, AccountProvider, AuthResult}};");
    println!();
    println!("let mut manager = AccountManager::new();");
    println!();
    println!("// Quick offline login (no credentials needed)");
    println!("let result = manager.quick_offline_login(\"TestPlayer\").await?;");
    println!();
    println!("// Get all stored accounts");
    println!("let accounts = manager.get_accounts().await?;");
    println!();
    println!("// Ensure token is valid");
    println!("for account in accounts {{");
    println!("    let fresh = manager.ensure_valid_token(&account).await?;");
    println!("}}");
    println!();
    println!("// Logout account");
    println!("manager.logout(\"username\", AccountProvider::Offline).await?;");
    println!("```");
    println!();

    println!("📚 Additional Resources:");
    println!("=========================");
    println!();
    println!("• API Documentation: Run `cargo doc --open`");
    println!("• Source Code: /src/account_management/");
    println!("• Tests: /tests/integration_tests.rs");
    println!("• Main Example: examples/account_management.rs (comprehensive)");
    println!();

    println!("✅ Ready to explore QLG Core!");
    println!("Run any of the provider examples above to get started.");

    Ok(())
}
