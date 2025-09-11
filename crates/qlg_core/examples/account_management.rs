//! # QLG Core Main Example Gateway
//!
//! This example serves as the main entry point and directs you to specific examples.
//! The comprehensive functionality has been split into focused examples for better organization.
//!
//! ## 🎯 **Available Examples**
//!
//! ### Provider-Specific Examples:
//! - `cargo run --example elyby_example` - ElyBy authentication
//! - `cargo run --example microsoft_example` - Microsoft OAuth authentication  
//! - `cargo run --example littleskin_example` - LittleSkin authentication
//! - `cargo run --example offline_example` - Offline/cracked authentication
//!
//! ### Management Examples:
//! - `cargo run --example management_example` - Account management, logout, token refresh
//! - `cargo run --example overview` - Comprehensive overview of all features
//!
//! ## 🚀 **Quick Start**
//!
//! 1. **For Testing**: `cargo run --example offline_example`
//! 2. **For Real Accounts**: `cargo run --example microsoft_example`
//! 3. **For Overview**: `cargo run --example overview`

use qlg_core::{AccountManager, AccountManagerTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("QLG Core Account Management - Example Gateway");
    println!("==============================================");
    println!();

    let manager = AccountManager::new();

    // Show quick overview
    println!("📋 Available Authentication Providers:");
    for provider in manager.supported_providers() {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        println!("  {} - Credentials: {}, OAuth: {}, Username-Only: {}", 
            provider, creds, oauth, username_only);
    }
    println!();

    // Show current accounts if any
    match manager.get_accounts().await {
        Ok(accounts) => {
            if !accounts.is_empty() {
                println!("📬 Currently stored accounts: {}", accounts.len());
                for account in accounts.iter().take(3) {
                    println!("  - {} ({})", account.display_username(), account.provider);
                }
                if accounts.len() > 3 {
                    println!("  ... and {} more", accounts.len() - 3);
                }
                println!();
            }
        }
        Err(_) => {} // Ignore keyring errors in gateway
    }

    println!("🎯 **Choose Your Example:**");
    println!();
    
    println!("**Provider-Specific Examples:**");
    println!("  cargo run --example elyby_example      # ElyBy authentication");
    println!("  cargo run --example microsoft_example  # Microsoft OAuth");
    println!("  cargo run --example littleskin_example # LittleSkin auth");
    println!("  cargo run --example offline_example    # Offline/testing");
    println!();
    
    println!("**Management Examples:**");
    println!("  cargo run --example management_example # Logout, tokens, keyring");
    println!("  cargo run --example overview          # Complete feature overview");
    println!();

    println!("💡 **Recommendations:**");
    println!();
    println!("🔰 **New to QLG Core?** Start here:");
    println!("  cargo run --example overview");
    println!();
    println!("🧪 **Want to test quickly?** Use offline accounts:");
    println!("  cargo run --example offline_example");
    println!();
    println!("🔐 **Need real authentication?** Try Microsoft:");
    println!("  cargo run --example microsoft_example");
    println!("  # Set TEST_OAUTH_FLOW = true in the file");
    println!();
    println!("🛠️  **Working with existing accounts?** Use management:");
    println!("  cargo run --example management_example");
    println!();

    println!("📚 **Documentation:**");
    println!("  cargo doc --open  # API documentation");
    println!("  # Source: /src/account_management/");
    println!();

    println!("✨ Each example is self-contained and focused on specific functionality!");

    Ok(())
}
