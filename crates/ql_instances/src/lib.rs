//! # A module for creating, managing and running Minecraft client instances
//!
//! This is a crate of
//! [Quantum Launcher](https://mrmayman.github.io/quantumlauncher)
//! for dealing with many operations of Minecraft instances.
//!
//! **Not recommended to use in your own projects!**
//!
//! This module contains functions to:
//! - Create an instance
//! - Launch the instance
//! - Update the launcher
//! - Read logs
//! - List versions available for download
//! - Authenticate with Microsoft Accounts
//!
//! We use [BetterJSONS](https://github.com/MCPHackers/BetterJSONs)
//! and [LaunchWrapper](https://github.com/MCPHackers/LaunchWrapper) for:
//! - Better platform compatibility (ARM, 32-bit, etc.)
//! - Many bugfixes
//! - Skin, Sound, Auth fixes for old versions
//! - Skin support from `ely.by` and `littleskin`
//!
//! # A rant about natives
//!
//! (probably not relevant for you)
//!
//! ## What are natives?
//! Natives are libraries that are platform-specific.
//! They are used by Minecraft to interface with the operating system.
//!
//! ## Types of natives
//! - `natives: *` - These are part of the main library
//!   but have a separate jar file included that is extracted to
//!   the `natives` folder.
//! - `name: *-natives-*` - They are a separate library
//!   whose jar file is extracted to the `natives` folder.
//! - `classifiers: *` - Once again, part of main library
//!   but have separate jar files for each OS. Just formatted
//!   differently in the json.
//!
//! For whatever reason, natives are arranged in an
//! **extremely** messy way in the JSONs.
//!
//! The library downloader is also extremely fragile and messy.
//! It's designed to cope with real world conditions,
//! but may not be ideal. Feel free to report bugs if found.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

pub mod auth;
mod download;
mod instance;
mod json_profiles;
mod launcher_update_detector;

pub use download::{create_instance, repeat_stage, DownloadError};
pub use instance::{launch::launch, list_versions::list_versions, notes};
pub use launcher_update_detector::{
    check_for_launcher_updates, install_launcher_update, UpdateCheckInfo, UpdateError,
};
pub use ql_core::jarmod;
pub use ql_java_handler::delete_java_installs;

pub use auth::token_store::TokenStorageMethod;

/// Set the global token storage method.
pub fn set_token_storage_method(method: TokenStorageMethod) {
    method.set_global();
}

/// Initialize the encrypted token store with a new password.
///
/// This should be called when user sets up encrypted storage for the first time.
pub fn initialize_encrypted_store(
    password: &str,
) -> Result<(), auth::token_store::TokenStoreError> {
    auth::encrypted_store::initialize_new(password)?;
    TokenStorageMethod::EncryptedFile.set_global();
    Ok(())
}

/// Unlock the encrypted token store with the given password.
pub fn unlock_encrypted_store(password: &str) -> Result<(), auth::token_store::TokenStoreError> {
    auth::encrypted_store::unlock(password)?;
    Ok(())
}

/// Check if encrypted store needs to be unlocked.
#[must_use]
pub fn encrypted_store_needs_unlock() -> bool {
    auth::token_store::needs_unlock()
}

/// Check if encrypted store file exists.
#[must_use]
pub fn encrypted_store_file_exists() -> bool {
    auth::token_store::encrypted_file_exists()
}

use semver::{BuildMetadata, Prerelease};

const LAUNCHER_VERSION: semver::Version = semver::Version {
    major: 0,
    minor: 5,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::EMPTY,
};
