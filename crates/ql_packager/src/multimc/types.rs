//! Type definitions for MultiMC instance format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main MultiMC package manifest (mmc-pack.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmcPack {
    /// Format version, typically 1
    pub format_version: i32,
    /// List of components (Minecraft, loaders, etc.)
    pub components: Vec<MmcComponent>,
}

/// A component in the MultiMC pack (Minecraft, Forge, Fabric, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmcComponent {
    /// Unique identifier (e.g., "net.minecraft", "net.minecraftforge")
    pub uid: String,
    /// Version string (optional for some components like jar mods)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_name: Option<String>,
    /// Cached version (sometimes duplicated, used as fallback if version is None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_version: Option<String>,
    /// Whether this component is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub important: Option<bool>,
    /// Dependency resolution hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_only: Option<bool>,
    /// Custom cached requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_requires: Option<Vec<MmcRequirement>>,
    /// Custom cached conflicts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_conflicts: Option<Vec<MmcRequirement>>,
}

/// Component requirement or conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcRequirement {
    pub uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggests: Option<String>,
}

/// Instance configuration (instance.cfg)
#[derive(Debug, Clone)]
pub struct MmcInstanceCfg {
    pub instance_type: String,
    pub name: String,
    pub icon_key: Option<String>,
    pub notes: Option<String>,
    pub jvm_args: Option<String>,
    pub java_path: Option<String>,
    pub launch_wrapper_command: Option<String>,
    pub override_commands: Option<bool>,
    pub override_console: Option<bool>,
    pub override_java_args: Option<bool>,
    pub override_java_location: Option<bool>,
    pub override_memory: Option<bool>,
    pub override_window: Option<bool>,
    pub max_memory: Option<u32>,
    pub min_memory: Option<u32>,
    pub permgen: Option<u32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub instance_group: Option<String>,
    pub last_launch_time: Option<i64>,
    pub total_time_played: Option<i64>,
}

/// Patch configuration for loaders and jar mods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmcPatch {
    /// Format version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_version: Option<i32>,
    /// Unique ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    /// Name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Main class
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    /// Minecraft arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    /// Libraries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<Vec<MmcLibrary>>,
    /// Jar mods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jar_mods: Option<Vec<MmcJarMod>>,
    /// Asset index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<MmcAssetIndex>,
    /// Main jar override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_jar: Option<MmcMainJar>,
    /// Custom traits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traits: Option<Vec<String>>,
    /// Order priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

/// Library entry in patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcLibrary {
    /// Maven name (group:artifact:version)
    pub name: String,
    /// Download URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Downloads info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<MmcDownloads>,
    /// Natives
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,
    /// Extract rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<MmcExtract>,
    /// Rules for conditional loading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<MmcRule>>,
}

/// Download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<MmcArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, MmcArtifact>>,
}

/// Artifact download info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
}

/// Extraction rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcExtract {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

/// Conditional rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcRule {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<MmcOs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

/// OS matcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcOs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

/// Jar mod entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmcJarMod {
    #[serde(rename = "MMC-displayname")]
    pub display_name: String,
    #[serde(rename = "MMC-filename")]
    pub filename: String,
}

/// Asset index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcAssetIndex {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Main jar override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcMainJar {
    #[serde(rename = "MMC-filename")]
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Path mapping (pathmap.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcPathMap {
    #[serde(flatten)]
    pub mappings: HashMap<String, String>,
}

/// Instance groups (instgroups.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmcInstGroups {
    pub groups: Vec<String>,
}

impl Default for MmcPack {
    fn default() -> Self {
        Self {
            format_version: 1,
            components: Vec::new(),
        }
    }
}

impl Default for MmcInstanceCfg {
    fn default() -> Self {
        Self {
            instance_type: "OneSix".to_string(),
            name: "Minecraft Instance".to_string(),
            icon_key: Some("default".to_string()),
            notes: None,
            jvm_args: None,
            java_path: None,
            launch_wrapper_command: None,
            override_commands: None,
            override_console: None,
            override_java_args: None,
            override_java_location: None,
            override_memory: None,
            override_window: None,
            max_memory: None,
            min_memory: None,
            permgen: None,
            window_width: None,
            window_height: None,
            instance_group: None,
            last_launch_time: None,
            total_time_played: None,
        }
    }
}
