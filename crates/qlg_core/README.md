# QLG Core - Account Management

A clean, trait-based interface for account management in Quantum Launcher. This crate provides easy-to-use abstractions for authentication with various Minecraft account providers.

## Features

- **Multiple Providers**: Support for Microsoft, ElyBy, and LittleSkin accounts
- **Async Support**: Full async/await support with Tokio
- **Secure Storage**: Automatic credential storage using system keyring
- **Token Management**: Automatic token refresh and validation

## Supported Account Types

| Provider   | Credentials Auth | OAuth Auth | Username Only | Premium Required |
| ---------- | ---------------- | ---------- | ------------- | ---------------- |
| Microsoft  | ❌               | ✅         | ❌            | ✅               |
| ElyBy      | ✅               | ❌         | ❌            | ❌               |
| LittleSkin | ✅               | ✅         | ❌            | ❌               |
| Offline    | ❌               | ❌         | ✅            | ❌               |

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
qlg_core = { path = "../qlg_core" }
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use qlg_core::{AccountManager, AccountProvider, LoginCredentials, AuthResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Login with username and password (ElyBy example)
    let result = manager.quick_login(
        AccountProvider::ElyBy,
        "your_username",
        "your_password"
    ).await?;

    match result {
        AuthResult::Success(account) => {
            println!("Welcome, {}!", account.display_name);
            println!("UUID: {}", account.uuid);
        }
        AuthResult::RequiresTwoFactor => {
            println!("Please provide 2FA code");
        }
        AuthResult::Failed(error) => {
            println!("Login failed: {}", error);
        }
    }

    Ok(())
}
```

### Offline Login (Username Only)

````rust
use qlg_core::{AccountManager, AccountProvider, AuthResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Simple offline login for cracked accounts
    let result = manager.quick_offline_login("TestPlayer").await?;

    match result {
        AuthResult::Success(account) => {
            println!("Offline account created: {}", account.display_username());
            println!("UUID: {}", account.uuid);
        }
        AuthResult::Failed(error) => {
            println!("Invalid username: {}", error);
        }
        _ => unreachable!("Offline accounts don't use 2FA"),
    }

    Ok(())
}
```### Microsoft OAuth Login

```rust
use qlg_core::{AccountManager, AccountProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Start OAuth flow
    let oauth_flow = manager.start_oauth_login(AccountProvider::Microsoft).await?;

    println!("Visit: {}", oauth_flow.verification_uri);
    println!("Enter code: {}", oauth_flow.user_code);

    // Wait for user to complete authentication
    let result = manager.complete_oauth_login(
        AccountProvider::Microsoft,
        &oauth_flow.device_code
    ).await?;

    if let AuthResult::Success(account) = result {
        println!("Microsoft login successful: {}", account.display_name);
    }

    Ok(())
}
````

### Account Management

```rust
use qlg_core::{AccountManager, AccountProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AccountManager::new();

    // Get all stored accounts
    let accounts = manager.get_accounts().await?;
    for account in accounts {
        println!("Account: {} ({})", account.display_username(), account.provider);

        // Ensure token is valid
        let updated_account = manager.ensure_valid_token(&account).await?;
        println!("Token valid: {}", updated_account.access_token.is_some());
    }

    // Logout an account
    manager.logout("username", AccountProvider::ElyBy).await?;

    Ok(())
}
```

## Architecture

The crate is designed around several key traits:

### `AuthProvider`

Individual authentication provider implementations (Microsoft, ElyBy, LittleSkin).

### `AccountManager`

Main orchestrator that manages multiple providers and provides a unified interface.

### `CredentialStore`

Handles secure storage and retrieval of refresh tokens and credentials.

## Error Handling

The crate uses a comprehensive error system:

```rust
use qlg_core::AccountError;

match manager.quick_login(provider, username, password).await {
    Ok(AuthResult::Success(account)) => {
        // Handle success
    }
    Ok(AuthResult::RequiresTwoFactor) => {
        // Handle 2FA requirement
    }
    Ok(AuthResult::Failed(msg)) => {
        // Handle authentication failure
    }
    Err(AccountError::Network(e)) => {
        // Handle network errors
    }
    Err(AccountError::Keyring(e)) => {
        // Handle credential storage errors
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Advanced Usage

### Custom Credential Store

You can implement your own credential storage:

```rust
use qlg_core::{CredentialStore, AccountProvider, AccountError};
use async_trait::async_trait;

struct CustomCredentialStore;

#[async_trait]
impl CredentialStore for CustomCredentialStore {
    async fn store_refresh_token(&self, username: &str, provider: AccountProvider, token: &str) -> Result<(), AccountError> {
        // Your custom storage logic
        Ok(())
    }

    async fn get_refresh_token(&self, username: &str, provider: AccountProvider) -> Result<String, AccountError> {
        // Your custom retrieval logic
        Ok("token".to_string())
    }

    // ... implement other methods
}

// Use with manager
let manager = AccountManager::with_credential_store(Arc::new(CustomCredentialStore));
```

### Provider Capabilities

Check what each provider supports:

```rust
let manager = AccountManager::new();

for provider in manager.supported_providers() {
    let (credentials_auth, oauth_auth) = manager.provider_capabilities(provider)?;
    println!("{}: credentials={}, oauth={}", provider, credentials_auth, oauth_auth);
}
```

## Integration with Existing Code

This crate is designed to work alongside the existing `ql_instances` crate. It provides a cleaner interface while reusing the underlying authentication logic.

### Converting to/from ql_instances types

The providers internally convert between the new types and existing `ql_instances::auth::AccountData` types, so you can gradually migrate existing code.

## Thread Safety

All components are designed to be thread-safe:

- `AccountManager` can be shared across threads
- All operations are async and non-blocking
- Credential storage is handled safely

## Examples

See the `examples/` directory for more comprehensive examples:

- `examples/usage.rs` - Complete usage examples
- Basic login flows
- OAuth flows
- Account management
- Error handling

## Future Enhancements

Planned features for future versions:

- [ ] Session persistence
- [ ] Account switching UI helpers
- [ ] More authentication providers
- [ ] Offline mode support
- [ ] Account metadata storage
- [ ] Event-driven architecture for UI integration

## Contributing

This crate is part of the Quantum Launcher project. When contributing:

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation
4. Ensure backward compatibility where possible

## License

Same as Quantum Launcher main project.
