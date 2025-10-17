//! MultiMC Instance Import/Export
//!
//! This module provides comprehensive support for importing and exporting
//! MultiMC/PrismLauncher instances, including:
//! - Full instance configuration (instance.cfg)
//! - Component management (mmc-pack.json)
//! - Jar mods and patches
//! - Loader mods (Forge, Fabric, Quilt, NeoForge)
//! - Custom icons and instance groups
//! - Java configuration and JVM arguments

pub mod export;
pub mod import;
pub mod legacy;
pub mod types;

pub use export::export_to_multimc;
pub use import::import_from_multimc;
pub use types::*;

// Re-export legacy import function for backward compatibility
pub use legacy::import as legacy_import;
