use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{InstanceSelection, IntoIoError, IntoJsonError, JsonFileError};

/// Defines how instance Java arguments should interact with global Java arguments
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JavaArgsMode {
    /// Use global arguments only if instance arguments are empty,
    /// as a *fallback*.
    #[serde(rename = "fallback")]
    Fallback,
    /// Disable global arguments entirely,
    /// **only** use instance arguments
    #[serde(rename = "disable")]
    Disable,
    /// Combine global arguments with instance arguments,
    /// using both together.
    #[serde(rename = "combine")]
    #[default]
    Combine,
}

/// SSL Certificate trust store configuration
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SslTrustStoreType {
    /// Use default Java trust store
    #[serde(rename = "default")]
    #[default]
    Default,
    /// Use Windows Certificate Store (Windows only)
    #[serde(rename = "windows-root")]
    WindowsRoot,
    /// Use system keychain (macOS only)  
    #[serde(rename = "keychain")]
    Keychain,
    /// Use custom trust store file
    #[serde(rename = "custom")]
    Custom,
}

impl JavaArgsMode {
    pub const ALL: &[Self] = &[Self::Combine, Self::Disable, Self::Fallback];

    pub fn get_description(self) -> &'static str {
        match self {
            JavaArgsMode::Fallback => "Use global arguments only when instance has no arguments",
            JavaArgsMode::Disable => "No global arguments are applied",
            JavaArgsMode::Combine => {
                "Global arguments are combined with instance arguments (default)"
            }
        }
    }
}

impl SslTrustStoreType {
    pub const ALL: &[Self] = &[Self::Default, Self::WindowsRoot, Self::Keychain, Self::Custom];

    pub fn get_description(self) -> &'static str {
        match self {
            SslTrustStoreType::Default => "Use default Java trust store",
            SslTrustStoreType::WindowsRoot => "Use Windows Certificate Store (Windows only)",
            SslTrustStoreType::Keychain => "Use system keychain (macOS only)",
            SslTrustStoreType::Custom => "Use custom trust store file",
        }
    }

    /// Returns true if this trust store type is supported on the current platform
    pub fn is_supported(self) -> bool {
        match self {
            SslTrustStoreType::Default | SslTrustStoreType::Custom => true,
            SslTrustStoreType::WindowsRoot => cfg!(target_os = "windows"),
            SslTrustStoreType::Keychain => cfg!(target_os = "macos"),
        }
    }

    /// Returns the Java system property value for this trust store type
    pub fn get_java_property(self) -> Option<String> {
        match self {
            SslTrustStoreType::Default => None,
            SslTrustStoreType::WindowsRoot => Some("Windows-ROOT".to_string()),
            SslTrustStoreType::Keychain => Some("KeychainStore".to_string()),
            SslTrustStoreType::Custom => None, // Requires custom path
        }
    }
}

impl std::fmt::Display for JavaArgsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JavaArgsMode::Fallback => write!(f, "Fallback"),
            JavaArgsMode::Disable => write!(f, "Disable"),
            JavaArgsMode::Combine => write!(f, "Combine (default)"),
        }
    }
}

impl std::fmt::Display for SslTrustStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SslTrustStoreType::Default => write!(f, "Default"),
            SslTrustStoreType::WindowsRoot => write!(f, "Windows Certificate Store"),
            SslTrustStoreType::Keychain => write!(f, "macOS Keychain"),
            SslTrustStoreType::Custom => write!(f, "Custom Trust Store"),
        }
    }
}

/// Configuration for a specific instance.
/// Not to be confused with [`crate::json::VersionDetails`]. That one
/// is launcher agnostic data provided from mojang, this one is
/// Quantum Launcher specific information.
///
/// Stored in:
/// - Client: `QuantumLauncher/instances/<instance_name>/config.json`
/// - Server: `QuantumLauncher/servers/<instance_name>/config.json`
///
/// See the documentation of each field for more information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfigJson {
    /// **Default: `"Vanilla"`**
    ///
    /// Can be one of:
    /// - `"Vanilla"` (unmodded)
    /// - `"Fabric"`
    /// - `"Forge"`
    /// - `"OptiFine"`
    /// - `"Quilt"`
    /// - `"NeoForge"`
    pub mod_type: String,
    /// If you want to use your own Java installation
    /// instead of the auto-installed one, specify
    /// the path to the `java` executable here.
    pub java_override: Option<String>,
    /// The amount of RAM in megabytes the instance should have.
    pub ram_in_mb: usize,
    /// **Default: `true`**
    ///
    /// - `true` (default): Show log output in launcher.
    ///   May not show all log output, especially during a crash.
    /// - `false`: Print raw, unformatted log output to the console (stdout).
    ///   This is useful for debugging, but may be hard to read.
    pub enable_logger: Option<bool>,
    /// This is an optional list of additional
    /// arguments to pass to Java.
    pub java_args: Option<Vec<String>>,
    /// This is an optional list of additional
    /// arguments to pass to the game.
    pub game_args: Option<Vec<String>>,
    /// DEPRECATED in v0.4.2
    ///
    /// This used to indicate whether a version
    /// was downloaded from Omniarchive instead
    /// of Mojang, in Quantum Launcher
    /// v0.3.1 - v0.4.1
    #[deprecated(since = "0.4.2", note = "migrated to BetterJSONs, so no longer needed")]
    pub omniarchive: Option<serde_json::Value>,
    /// **Default: `false`**
    ///
    /// - `true`: the instance is a classic server.
    /// - `false` (default): the instance is a client
    ///   or a non-classic server (alpha, beta, release).
    ///
    /// This is stored because classic servers:
    /// - Are downloaded differently (zip file to extract)
    /// - Cannot be stopped by sending a `stop` command.
    ///   (need to kill the process)
    pub is_classic_server: Option<bool>,
    /// **`false` for client instances, `true` for server installations**
    ///
    /// Added in v0.4.2
    pub is_server: Option<bool>,
    /// **Client Only**
    ///
    /// If true, then the Java Garbage Collector
    /// will be modified through launch arguments,
    /// for *different* performance.
    ///
    /// **Default: `false`**
    ///
    /// This doesn't specifically improve performance,
    /// in fact from my testing it worsens them?:
    ///
    /// - Without these args I got 110-115 FPS average on vanilla
    ///   Minecraft 1.20 in a new world.
    ///
    /// - With these args I got 105-110 FPS. So... yeah they aren't
    ///   doing the job for me.
    ///
    /// But in different workloads this might improve performance.
    ///
    /// # Arguments
    ///
    /// The G1 garbage collector will be used.
    /// Here are the specific arguments.
    ///
    /// - `-XX:+UnlockExperimentalVMOptions`
    /// - `-XX:+UseG1GC`
    /// - `-XX:G1NewSizePercent=20`
    /// - `-XX:G1ReservePercent=20`
    /// - `-XX:MaxGCPauseMillis=50`
    /// - `-XX:G1HeapRegionSize=32M`
    pub do_gc_tuning: Option<bool>,
    /// **Client Only**
    ///
    /// Whether to close the launcher upon
    /// starting the game.
    ///
    /// **Default: `false`**
    ///
    /// This keeps *just the game* running
    /// after you open it. However:
    /// - The impact of keeping the launcher open
    ///   is downright **negligible**. Quantum Launcher
    ///   is **very** lightweight. You won't feel any
    ///   difference even on slow computers
    /// - By doing this you lose access to easy log viewing
    ///   and the ability to easily kill the game process if stuck
    ///
    /// Ultimately if you want one less icon in your taskbar then go ahead.
    pub close_on_start: Option<bool>,

    pub global_settings: Option<GlobalSettings>,

    /// Controls how this instance's Java arguments interact with global Java arguments.
    ///
    /// **Default: `JavaArgsMode::Fallback`**
    ///
    /// - `Fallback`: Use global args only when instance has no meaningful args (backward compatible)
    /// - `Override`: Instance args completely replace global args (ignore global when instance has args)
    /// - `Combine`: Global args are prepended to instance args (both are used together)
    pub java_args_mode: Option<JavaArgsMode>,

    /// **SSL Certificate Configuration**
    ///
    /// Configures which certificate trust store Java should use for SSL/TLS connections.
    /// This helps fix SSL connection issues with Minecraft servers and mod downloads.
    ///
    /// **Default: `SslTrustStoreType::Default`**
    ///
    /// - `Default`: Use Java's default trust store
    /// - `WindowsRoot`: Use Windows Certificate Store (fixes SSL issues on Windows)
    /// - `Keychain`: Use macOS Keychain (for macOS users)
    /// - `Custom`: Use a custom trust store file (requires `ssl_trust_store_path`)
    pub ssl_trust_store_type: Option<SslTrustStoreType>,

    /// **Custom SSL Trust Store Path**
    ///
    /// Path to a custom trust store file (JKS, PKCS12, etc.) when using
    /// `ssl_trust_store_type = Custom`.
    ///
    /// Example: `/path/to/custom-truststore.jks`
    pub ssl_trust_store_path: Option<String>,

    /// **Custom SSL Trust Store Password**
    ///
    /// Password for the custom trust store file. Only used when
    /// `ssl_trust_store_type = Custom`.
    pub ssl_trust_store_password: Option<String>,
}

impl InstanceConfigJson {
    /// Returns a String containing the Java argument to
    /// allocate the configured amount of RAM.
    #[must_use]
    pub fn get_ram_argument(&self) -> String {
        format!("-Xmx{}M", self.ram_in_mb)
    }

    /// Loads the launcher-specific instance configuration from disk,
    /// based on a path to the root of the instance directory.
    ///
    /// # Errors
    /// - `dir`/`config.json` doesn't exist or isn't a file
    /// - `config.json` file couldn't be loaded
    /// - `config.json` couldn't be parsed into valid JSON
    pub async fn read_from_dir(dir: &Path) -> Result<Self, JsonFileError> {
        let config_json_path = dir.join("config.json");
        let config_json = tokio::fs::read_to_string(&config_json_path)
            .await
            .path(config_json_path)?;
        Ok(serde_json::from_str(&config_json).json(config_json)?)
    }

    /// Loads the launcher-specific instance configuration from disk,
    /// based on a specific `InstanceSelection`
    ///
    /// # Errors
    /// - `config.json` file couldn't be loaded
    /// - `config.json` couldn't be parsed into valid JSON
    pub async fn read(instance: &InstanceSelection) -> Result<Self, JsonFileError> {
        Self::read_from_dir(&instance.get_instance_path()).await
    }

    /// Saves the launcher-specific instance configuration to disk,
    /// based on a path to the root of the instance directory.
    ///
    /// # Errors
    /// - `config.json` file couldn't be written to
    pub async fn save_to_dir(&self, dir: &Path) -> Result<(), JsonFileError> {
        let config_json_path = dir.join("config.json");
        let config_json = serde_json::to_string_pretty(self).json_to()?;
        tokio::fs::write(&config_json_path, config_json)
            .await
            .path(config_json_path)?;
        Ok(())
    }

    /// Saves the launcher-specific instance configuration to disk,
    /// based on a specific `InstanceSelection`
    ///
    /// # Errors
    /// - `config.json` file couldn't be written to
    /// - `self` couldn't be serialized into valid JSON
    pub async fn save(&self, instance: &InstanceSelection) -> Result<(), JsonFileError> {
        self.save_to_dir(&instance.get_instance_path()).await
    }

    #[must_use]
    pub fn get_window_size(&self, global: Option<&GlobalSettings>) -> (Option<u32>, Option<u32>) {
        let local = self.global_settings.as_ref();
        (
            local
                .and_then(|n| n.window_width)
                .or(global.and_then(|n| n.window_width)),
            local
                .and_then(|n| n.window_height)
                .or(global.and_then(|n| n.window_height)),
        )
    }

    /// Gets Java arguments with global fallback/combination support.
    ///
    /// The behavior depends on the instance's `java_args_mode`:
    /// - `Fallback`: Returns instance args if meaningful, otherwise global args
    /// - `Override`: Returns instance args only (ignores global even if instance is empty)
    /// - `Combine`: Returns global args + instance args (global first)
    ///
    /// Returns an empty vector if no arguments should be used.
    #[must_use]
    pub fn get_java_args(&self, global_args: &[String]) -> Vec<String> {
        let mode = self
            .java_args_mode
            .as_ref()
            .unwrap_or(&JavaArgsMode::Combine);
        let instance_args = self.java_args.as_ref();

        let has_meaningful_instance_args =
            instance_args.map_or(false, |args| args.iter().any(|arg| !arg.trim().is_empty()));

        match mode {
            // Use instance if meaningful, otherwise global
            JavaArgsMode::Fallback => {
                if has_meaningful_instance_args {
                    instance_args.unwrap().clone()
                } else {
                    global_args.to_owned()
                }
            }
            // Use instance args only, ignore global completely
            JavaArgsMode::Disable => {
                if has_meaningful_instance_args {
                    instance_args.unwrap().clone()
                } else {
                    Vec::new()
                }
            }
            // Combine both instance and global args
            JavaArgsMode::Combine => {
                let mut combined = Vec::new();
                combined.extend(global_args.iter().filter(|n| !n.trim().is_empty()).cloned());
                if has_meaningful_instance_args {
                    combined.extend(instance_args.unwrap().iter().cloned());
                }

                combined
            }
        }
    }

    /// Gets SSL-related Java arguments based on the configured trust store type.
    ///
    /// Returns a vector of Java system property arguments for SSL configuration.
    /// These arguments help fix SSL connection issues with Minecraft servers and mod downloads.
    /// Uses instance settings first, then falls back to global settings.
    #[must_use]
    pub fn get_ssl_java_args(&self, global_settings: Option<&GlobalSettings>) -> Vec<String> {
        // Use instance settings first, then fall back to global settings
        let trust_store_type = self
            .ssl_trust_store_type
            .or(global_settings.and_then(|g| g.ssl_trust_store_type))
            .unwrap_or(SslTrustStoreType::Default);

        let trust_store_path = self
            .ssl_trust_store_path
            .as_ref()
            .or(global_settings.and_then(|g| g.ssl_trust_store_path.as_ref()));

        let trust_store_password = self
            .ssl_trust_store_password
            .as_ref()
            .or(global_settings.and_then(|g| g.ssl_trust_store_password.as_ref()));

        let mut ssl_args = Vec::new();

        match trust_store_type {
            SslTrustStoreType::Default => {
                // No additional arguments needed for default trust store
            }
            SslTrustStoreType::WindowsRoot => {
                if cfg!(target_os = "windows") {
                    ssl_args.push("-Djavax.net.ssl.trustStoreType=Windows-ROOT".to_string());
                }
            }
            SslTrustStoreType::Keychain => {
                if cfg!(target_os = "macos") {
                    ssl_args.push("-Djavax.net.ssl.trustStoreType=KeychainStore".to_string());
                }
            }
            SslTrustStoreType::Custom => {
                if let Some(store_path) = trust_store_path {
                    if !store_path.trim().is_empty() {
                        ssl_args.push(format!("-Djavax.net.ssl.trustStore={}", store_path));
                        
                        if let Some(password) = trust_store_password {
                            if !password.trim().is_empty() {
                                ssl_args.push(format!("-Djavax.net.ssl.trustStorePassword={}", password));
                            }
                        }
                        
                        // Try to detect trust store type from file extension
                        if store_path.to_lowercase().ends_with(".p12") || store_path.to_lowercase().ends_with(".pfx") {
                            ssl_args.push("-Djavax.net.ssl.trustStoreType=PKCS12".to_string());
                        } else if store_path.to_lowercase().ends_with(".jks") {
                            ssl_args.push("-Djavax.net.ssl.trustStoreType=JKS".to_string());
                        }
                    }
                }
            }
        }

        ssl_args
    }
}

/// Settings that can both be set on a per-instance basis
/// and also have a global default.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalSettings {
    /// **Client Only**
    ///
    /// Custom window **width** for Minecraft (in windowed mode).
    ///
    /// When set, this will launch Minecraft with a specific window width
    /// using the `--width` command line argument.
    pub window_width: Option<u32>,
    /// **Client Only**
    ///
    /// Custom window **height** for Minecraft (in windowed mode).
    ///
    /// When set, this will launch Minecraft with a specific window height
    /// using the `--height` command line argument.
    pub window_height: Option<u32>,
    
    /// **SSL Certificate Configuration (Global Default)**
    ///
    /// Global default for SSL trust store type. Can be overridden per-instance.
    /// This helps fix SSL connection issues with Minecraft servers and mod downloads.
    pub ssl_trust_store_type: Option<SslTrustStoreType>,

    /// **Global SSL Trust Store Path**
    ///
    /// Global default path to custom trust store file. Can be overridden per-instance.
    pub ssl_trust_store_path: Option<String>,

    /// **Global SSL Trust Store Password**
    ///
    /// Global default password for custom trust store. Can be overridden per-instance.
    pub ssl_trust_store_password: Option<String>,
}
