//! Example demonstrating credential security features
//! 
//! This example shows how the new secure credential handling prevents
//! password leaks in memory and debug output.

use qlg_core::{LoginCredentials, AccessToken, SecureString, AccountManager, AccountProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 QLG Core Security Demo");
    println!("========================\n");

    // 1. Demonstrate secure credential creation
    println!("1. Creating secure credentials...");
    let credentials = LoginCredentials::new(
        "demo_user",
        "super_secret_password_123",
        Some("987654".to_string())
    );

    // 2. Show that debug output is safe
    println!("2. Debug output (safe):");
    println!("   {:?}", credentials);
    println!("   ↑ Notice: passwords are <redacted>\n");

    // 3. Demonstrate secure string behavior
    println!("3. SecureString behavior:");
    let secure_str = SecureString::new("sensitive_information");
    println!("   Debug output: {:?}", secure_str);
    println!("   Length: {}", secure_str.len());
    println!("   ↑ Content is hidden, only metadata shown\n");

    // 4. Demonstrate access token security
    println!("4. AccessToken security:");
    let token = AccessToken::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.very_long_token_string");
    println!("   Debug output: {:?}", token);
    println!("   Preview: {}", token.preview());
    println!("   ↑ Only safe preview shown\n");

    // 5. Show how to safely access credentials when needed
    println!("5. Safe credential access:");
    println!("   Username: {}", credentials.username);
    // Note: We don't actually print the password for security
    println!("   Password length: {}", credentials.get_password().len());
    if let Some(totp) = credentials.get_totp_code() {
        println!("   TOTP length: {}", totp.len());
    }
    println!("   ↑ Actual secrets not printed\n");

    // 6. Demonstrate usage with account manager
    println!("6. Using with AccountManager:");
    let mut manager = AccountManager::new();
    
    // These methods now use secure credentials internally
    match manager.quick_login(AccountProvider::Offline, "demo_user", "demo_pass").await {
        Ok(result) => println!("   ✓ Login attempt successful: {:?}", result),
        Err(e) => println!("   ⚠ Login failed (expected for demo): {}", e),
    }

    // 7. Show provider capabilities
    println!("\n7. Provider security capabilities:");
    for &provider in &[AccountProvider::Microsoft, AccountProvider::ElyBy, 
                       AccountProvider::LittleSkin, AccountProvider::Offline] {
        let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
        println!("   {}: credentials={}, oauth={}, username_only={}", 
                 provider, creds, oauth, username_only);
    }

    println!("\n🔒 Security Features Demonstrated:");
    println!("   ✓ Automatic credential zeroization on drop");
    println!("   ✓ Safe debug output (no password leaks)");
    println!("   ✓ Controlled access to sensitive data");
    println!("   ✓ Memory-safe credential handling");
    println!("   ✓ Backward-compatible API");

    println!("\n🧪 Memory Safety:");
    println!("   • Passwords are automatically cleared from memory");
    println!("   • Debug logs won't contain sensitive information");
    println!("   • Core dumps won't expose credentials");
    println!("   • Timing attacks are harder due to secure handling");

    Ok(())
}
