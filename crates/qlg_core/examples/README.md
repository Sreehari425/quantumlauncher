# QLG Core Examples

This directory contains focused examples demonstrating different aspects of QLG Core's account management functionality.

## 🎯 Quick Start

**New to QLG Core?** Start here:

```bash
cargo run --example overview
```

**Want to test immediately?** Use offline accounts:

```bash
cargo run --example offline_example
```

## 📚 Available Examples

### 🚪 Gateway Example

- **`account_management.rs`** - Main entry point that directs you to specific examples

### 📖 Overview Example

- **`overview.rs`** - Comprehensive overview of all QLG Core features and capabilities

### 🔐 Provider-Specific Examples

- **`elyby_example.rs`** - ElyBy authentication (username/password)
- **`microsoft_example.rs`** - Microsoft OAuth authentication (device code flow)
- **`littleskin_example.rs`** - LittleSkin authentication (username/password + OAuth)
- **`offline_example.rs`** - Offline/cracked authentication (username-only)

### 🛠️ Management Example

- **`management_example.rs`** - Account management, logout, token refresh, keyring operations

## 🔧 Configuration

Each example has configuration constants at the top:

```rust
const TEST_WITH_REAL_CREDENTIALS: bool = false;  // Enable real auth testing
const TEST_OAUTH_FLOW: bool = false;             // Enable OAuth testing
const TEST_LOGOUT_EXAMPLES: bool = true;         // Enable logout testing
const TEST_TOKEN_REFRESH: bool = true;           // Enable token testing
```

## 🚀 Running Examples

```bash
# Gateway - shows all available examples
cargo run --example account_management

# Overview - comprehensive feature overview
cargo run --example overview

# Provider-specific examples
cargo run --example elyby_example
cargo run --example microsoft_example
cargo run --example littleskin_example
cargo run --example offline_example

# Management functionality
cargo run --example management_example
```

## 🎭 Provider Comparison

| Provider       | Auth Method         | Token Type       | Use Case            |
| -------------- | ------------------- | ---------------- | ------------------- |
| **Microsoft**  | OAuth 2.0           | Refresh → Access | Premium Minecraft   |
| **ElyBy**      | Username/Password   | Access           | Alternative auth    |
| **LittleSkin** | Credentials + OAuth | Access           | Chinese community   |
| **Offline**    | Username only       | None             | Development/Testing |

## ⚠️ Security Notes

- **Never commit real credentials** to version control
- Tokens are stored securely in OS keyring
- Logout removes stored tokens permanently
- Set `TEST_WITH_REAL_CREDENTIALS = true` only for local testing

## 📖 Integration Examples

### Basic Usage

```rust
use qlg_core::{AccountManager, AccountProvider, AuthResult};

let mut manager = AccountManager::new();

// Quick offline login (no credentials needed)
let result = manager.quick_offline_login("TestPlayer").await?;

// Get all stored accounts
let accounts = manager.get_accounts().await?;

// Ensure token is valid
for account in accounts {
    let fresh = manager.ensure_valid_token(&account).await?;
}

// Logout account
manager.logout("username", AccountProvider::Offline).await?;
```

### Provider Capabilities

```rust
let manager = AccountManager::new();

for provider in manager.supported_providers() {
    let (creds, oauth, username_only) = manager.provider_capabilities(provider)?;
    println!("{}: creds={}, oauth={}, username={}", provider, creds, oauth, username_only);
}
```

## 📚 Additional Resources

- **API Documentation**: `cargo doc --open`
- **Source Code**: `/src/account_management/`
- **Integration Tests**: `/tests/integration_tests.rs`
- **Crate Root**: [QLG Core README](../README.md)

## 💡 Example Recommendations

| Scenario                      | Recommended Example                   |
| ----------------------------- | ------------------------------------- |
| 🔰 First time using QLG Core  | `overview`                            |
| 🧪 Quick testing/development  | `offline_example`                     |
| 🔐 Premium Minecraft accounts | `microsoft_example`                   |
| 🌐 Alternative auth services  | `elyby_example`, `littleskin_example` |
| 🛠️ Managing existing accounts | `management_example`                  |
| 📋 Feature exploration        | `account_management` (gateway)        |

Each example is self-contained and focuses on specific functionality for better learning and reference!
