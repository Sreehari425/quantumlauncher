//! Error types for account management

/// Errors that can occur during account management
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Account doesn't own Minecraft")]
    NoMinecraftOwnership,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Provider not supported for this operation")]
    UnsupportedProvider,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rate limit exceeded, please try again later")]
    RateLimitExceeded,
}

/// Convenient type alias for Results in account management
pub type Result<T> = std::result::Result<T, AccountError>;
