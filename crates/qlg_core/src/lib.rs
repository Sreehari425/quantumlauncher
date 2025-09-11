//! # QLG Core - Account Management
//!
//! A clean, trait-based interface for account management in Quantum Launcher.
//! This crate provides easy-to-use abstractions for authentication with:
//! - Microsoft accounts (MSA)
//! - ElyBy accounts
//! - LittleSkin accounts
//!
//! ## Features
//! - Simple trait-based design
//! - Async support
//! - Secure credential storage
//! - Easy account switching
//! - Token refresh management

pub mod account_management;

pub use account_management::*;

// Ensure the trait is available for method calls
pub use account_management::AccountManagerTrait;

// Export security types for testing
pub use account_management::SecureString;
