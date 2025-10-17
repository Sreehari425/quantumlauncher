//! MultiMC Instance Import
//!
//! Handles importing MultiMC/PrismLauncher instances into QuantumLauncher format

use super::types::*;
use crate::{import::pipe_progress, import::OUT_OF, InstancePackageError};
use chrono::DateTime;
use ini::Ini;
use ql_core::{
    do_jobs, err, file_utils, info,
    jarmod::{JarMod, JarMods},
    json::{
        FabricJSON, InstanceConfigJson, Manifest, VersionDetails, V_1_12_2,
        V_OFFICIAL_FABRIC_SUPPORT,
    },
    pt, GenericProgress, InstanceSelection, IntoIoError, IntoJsonError, ListEntry, Loader,
};
use ql_mod_manager::loaders::fabric::just_get_a_version;
use std::{
    path::Path,
    sync::{mpsc::Sender, Arc, Mutex},
};
use tokio::fs;

/// Import a MultiMC/PrismLauncher instance from extracted directory
pub async fn import_from_multimc(
    download_assets: bool,
    temp_dir: &Path,
    sender: Option<Arc<Sender<GenericProgress>>>,
) -> Result<InstanceSelection, InstancePackageError> {
    info!("Importing MultiMC/PrismLauncher instance...");

    // Read mmc-pack.json
    let mmc_pack_path = temp_dir.join("mmc-pack.json");
    let mmc_pack_content = fs::read_to_string(&mmc_pack_path)
        .await
        .path(&mmc_pack_path)?;
    let mmc_pack: MmcPack =
        serde_json::from_str(&mmc_pack_content).json(mmc_pack_content.clone())?;

    // Read instance.cfg
    let cfg = read_instance_cfg(temp_dir).await?;

    // Create instance selection
    let instance_selection = InstanceSelection::new(&cfg.name, false);

    // Parse components and determine what to install
    let recipe = parse_components(&mmc_pack.components).await?;

    pt!("Instance: {}", cfg.name);
    pt!("Minecraft Version: {}", recipe.mc_version);
    if let Some(ref loader) = recipe.loader {
        pt!(
            "Loader: {:?} {}",
            loader,
            recipe.loader_version.as_deref().unwrap_or("latest")
        );
    }

    // Create base Minecraft instance
    create_minecraft_instance(
        download_assets,
        sender.clone(),
        &cfg.name,
        &recipe.mc_version,
    )
    .await?;

    // Install loader if specified
    if let Some(loader) = recipe.loader {
        install_loader(
            sender.clone(),
            &instance_selection,
            loader,
            recipe.loader_version.clone(),
        )
        .await?;
    }

    // Copy files from MultiMC instance
    copy_multimc_files(temp_dir, sender.clone(), &instance_selection).await?;

    // Import jar mods if present
    import_jar_mods(temp_dir, &instance_selection).await?;

    // Apply instance configuration
    apply_instance_config(&cfg, &instance_selection).await?;

    if recipe.force_vanilla_launch {
        force_main_class_override(&instance_selection, "net.minecraft.client.Minecraft").await?;
    }

    // Upgrade legacy-lwjgl3 library for LWJGL3 instances with custom jar mods
    if recipe.is_lwjgl3 && recipe.force_vanilla_launch {
        upgrade_legacy_lwjgl3(&instance_selection).await?;
    }

    // Import patches if present
    import_patches(temp_dir, &instance_selection).await?;

    info!("Finished importing MultiMC/PrismLauncher instance");
    Ok(instance_selection)
}

/// Recipe for creating an instance
#[derive(Debug, Clone)]
struct InstanceRecipe {
    mc_version: String,
    loader: Option<Loader>,
    loader_version: Option<String>,
    is_lwjgl3: bool,
    force_vanilla_launch: bool,
}

/// Parse MultiMC components and determine what to install
async fn parse_components(
    components: &[MmcComponent],
) -> Result<InstanceRecipe, InstancePackageError> {
    let mut recipe = InstanceRecipe {
        mc_version: "1.19.2".to_string(), // Default fallback
        loader: None,
        loader_version: None,
        is_lwjgl3: false,
        force_vanilla_launch: false,
    };

    for component in components {
        if component.uid.starts_with("custom.jarmod.") {
            recipe.force_vanilla_launch = true;
        }

        let name = component.cached_name.as_deref().unwrap_or(&component.uid);

        // Get version from either version or cached_version field
        let version = component
            .version
            .as_ref()
            .or(component.cached_version.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                pt!(
                    "Warning: Component {} has no version, using empty string",
                    component.uid
                );
                String::new()
            });

        match component.uid.as_str() {
            "net.minecraft" => {
                recipe.mc_version = version;
            }
            "net.minecraftforge" => {
                recipe.loader = Some(Loader::Forge);
                recipe.loader_version = Some(version);
            }
            "net.neoforged" => {
                recipe.loader = Some(Loader::Neoforge);
                recipe.loader_version = Some(version);
            }
            "net.fabricmc.fabric-loader" => {
                recipe.loader = Some(Loader::Fabric);
                recipe.loader_version = Some(version);
            }
            "org.quiltmc.quilt-loader" => {
                recipe.loader = Some(Loader::Quilt);
                recipe.loader_version = Some(version);
            }
            "org.lwjgl3" => {
                recipe.is_lwjgl3 = true;
            }
            "org.lwjgl" => {
                // Check if this is LWJGL 3 by version
                if version.starts_with('3') {
                    recipe.is_lwjgl3 = true;
                    pt!("Detected LWJGL 3 (version {})", version);
                }
                // Otherwise it's LWJGL 2, no action needed
            }
            "net.fabricmc.intermediary" => {
                // These are dependencies, no action needed
            }
            _ => {
                pt!("Unknown MultiMC component: {} ({})", name, component.uid);
            }
        }
    }

    // Handle LWJGL3 version adjustment for older Minecraft versions
    if recipe.is_lwjgl3 {
        recipe.adjust_for_lwjgl3().await?;
    }

    Ok(recipe)
}

impl InstanceRecipe {
    async fn adjust_for_lwjgl3(&mut self) -> Result<(), InstancePackageError> {
        // Check if this is an old version that needs LWJGL3 adjustment
        let manifest = Manifest::download().await?;
        if let Some(version) = manifest.find_name(&self.mc_version) {
            if let (Ok(release_time), Ok(cutoff)) = (
                DateTime::parse_from_rfc3339(&version.releaseTime),
                DateTime::parse_from_rfc3339(V_1_12_2),
            ) {
                if release_time <= cutoff {
                    self.mc_version.push_str("-lwjgl3");
                }
            }
        }
        Ok(())
    }
}

/// Read and parse instance.cfg
async fn read_instance_cfg(temp_dir: &Path) -> Result<MmcInstanceCfg, InstancePackageError> {
    let cfg_path = temp_dir.join("instance.cfg");
    let cfg_content = fs::read_to_string(&cfg_path).await.path(&cfg_path)?;
    let cfg_content = filter_bytearray(&cfg_content);
    let ini = Ini::load_from_str(&cfg_content)?;

    let get_value = |section: Option<&str>, key: &str| -> Option<String> {
        ini.get_from(section, key).map(|s| s.to_string())
    };

    let _get_section = |section: &str| -> Option<&ini::Properties> { ini.section(Some(section)) };

    Ok(MmcInstanceCfg {
        instance_type: get_value(Some("General"), "InstanceType")
            .unwrap_or_else(|| "OneSix".to_string()),
        name: get_value(Some("General"), "name")
            .or_else(|| get_value(None, "name"))
            .ok_or_else(|| {
                InstancePackageError::IniFieldMissing("General".to_string(), "name".to_string())
            })?,
        icon_key: get_value(Some("General"), "iconKey"),
        notes: get_value(Some("General"), "notes"),
        jvm_args: get_value(Some("General"), "JvmArgs"),
        java_path: get_value(Some("General"), "JavaPath"),
        launch_wrapper_command: get_value(Some("General"), "LaunchWrapperCommand"),
        override_commands: get_value(Some("General"), "OverrideCommands")
            .and_then(|v| v.parse().ok()),
        override_console: get_value(Some("General"), "OverrideConsole")
            .and_then(|v| v.parse().ok()),
        override_java_args: get_value(Some("General"), "OverrideJavaArgs")
            .and_then(|v| v.parse().ok()),
        override_java_location: get_value(Some("General"), "OverrideJavaLocation")
            .and_then(|v| v.parse().ok()),
        override_memory: get_value(Some("General"), "OverrideMemory").and_then(|v| v.parse().ok()),
        override_window: get_value(Some("General"), "OverrideWindow").and_then(|v| v.parse().ok()),
        max_memory: get_value(Some("General"), "MaxMemAlloc").and_then(|v| v.parse().ok()),
        min_memory: get_value(Some("General"), "MinMemAlloc").and_then(|v| v.parse().ok()),
        permgen: get_value(Some("General"), "PermGen").and_then(|v| v.parse().ok()),
        window_width: get_value(Some("General"), "LaunchMaximized")
            .and_then(|v| {
                if v == "false" {
                    get_value(Some("General"), "MinecraftWinWidth")
                } else {
                    None
                }
            })
            .and_then(|v| v.parse().ok()),
        window_height: get_value(Some("General"), "LaunchMaximized")
            .and_then(|v| {
                if v == "false" {
                    get_value(Some("General"), "MinecraftWinHeight")
                } else {
                    None
                }
            })
            .and_then(|v| v.parse().ok()),
        instance_group: get_value(Some("General"), "InstanceGroup"),
        last_launch_time: get_value(Some("General"), "lastLaunchTime").and_then(|v| v.parse().ok()),
        total_time_played: get_value(Some("General"), "totalTimePlayed")
            .and_then(|v| v.parse().ok()),
    })
}

/// Create base Minecraft instance
async fn create_minecraft_instance(
    download_assets: bool,
    sender: Option<Arc<Sender<GenericProgress>>>,
    instance_name: &str,
    version: &str,
) -> Result<(), InstancePackageError> {
    let version = ListEntry {
        name: version.to_string(),
        is_classic_server: false,
    };
    let (d_send, d_recv) = std::sync::mpsc::channel();
    if let Some(sender) = sender.clone() {
        std::thread::spawn(move || {
            pipe_progress(d_recv, &sender);
        });
    }
    ql_instances::create_instance(
        instance_name.to_owned(),
        version,
        Some(d_send),
        download_assets,
    )
    .await?;
    Ok(())
}

/// Install loader (Forge, Fabric, Quilt, NeoForge)
async fn install_loader(
    sender: Option<Arc<Sender<GenericProgress>>>,
    instance_selection: &InstanceSelection,
    loader: Loader,
    version: Option<String>,
) -> Result<(), InstancePackageError> {
    match loader {
        Loader::Fabric | Loader::Quilt => {
            install_fabric(
                sender.as_deref(),
                instance_selection,
                version,
                matches!(loader, Loader::Quilt),
            )
            .await?;
        }
        Loader::Forge => {
            install_forge(sender, instance_selection, version, false).await?;
        }
        Loader::Neoforge => {
            install_forge(sender, instance_selection, version, true).await?;
        }
        _ => {
            err!("Unsupported loader: {:?}", loader);
        }
    }
    Ok(())
}

/// Install Fabric or Quilt loader
async fn install_fabric(
    sender: Option<&Sender<GenericProgress>>,
    instance_selection: &InstanceSelection,
    version: Option<String>,
    is_quilt: bool,
) -> Result<(), InstancePackageError> {
    let version_json = VersionDetails::load(instance_selection).await?;

    // For 1.14+ use normal Fabric installation
    if !version_json.is_before_or_eq(V_OFFICIAL_FABRIC_SUPPORT) {
        ql_mod_manager::loaders::fabric::install(
            version,
            instance_selection.clone(),
            sender,
            is_quilt,
        )
        .await?;
        return Ok(());
    }

    // For older versions, use custom implementation
    pt!("Using legacy Fabric installation for pre-1.14 version");
    let loader_version = if let Some(v) = version {
        v
    } else {
        just_get_a_version(instance_selection, is_quilt).await?
    };

    let url = format!(
        "https://{}/versions/loader/1.14.4/{}/profile/json",
        if is_quilt {
            "meta.quiltmc.org/v3"
        } else {
            "meta.fabricmc.net/v2"
        },
        loader_version
    );

    let fabric_json_text = file_utils::download_file_to_string(&url, false).await?;
    let fabric_json: FabricJSON =
        serde_json::from_str(&fabric_json_text).json(fabric_json_text.clone())?;

    let instance_path = instance_selection.get_instance_path();
    let libraries_dir = instance_path.join("libraries");

    info!("Installing Fabric libraries for legacy version:");
    let i = Mutex::new(0);
    let len = fabric_json.libraries.len();
    do_jobs(fabric_json.libraries.iter().map(|library| async {
        if library.name.starts_with("net.fabricmc:intermediary") {
            return Ok::<_, InstancePackageError>(());
        }
        let path_str = library.get_path();
        let url = library.get_url();
        let path = libraries_dir.join(&path_str);

        let parent_dir = path
            .parent()
            .ok_or(InstancePackageError::PathBufParent(path.clone()))?;
        tokio::fs::create_dir_all(parent_dir)
            .await
            .path(parent_dir)?;
        file_utils::download_file_to_path(&url, false, &path).await?;

        {
            let mut i = i.lock().unwrap();
            *i += 1;
            pt!(
                "({i}/{len}) {}\n    Path: {path_str}\n    Url: {url}",
                library.name
            );
            if let Some(sender) = sender {
                _ = sender.send(GenericProgress {
                    done: *i,
                    total: len,
                    message: Some(format!("Installing fabric: library {}", library.name)),
                    has_finished: false,
                });
            }
        }

        Ok(())
    }))
    .await?;

    // Update config
    let mut config = InstanceConfigJson::read(instance_selection).await?;
    config.main_class_override = Some(fabric_json.mainClass.clone());
    config.mod_type = if is_quilt { "Quilt" } else { "Fabric" }.to_owned();
    config.save(instance_selection).await?;

    // Save fabric.json
    let fabric_json_path = instance_path.join("fabric.json");
    tokio::fs::write(&fabric_json_path, &fabric_json_text)
        .await
        .path(&fabric_json_path)?;

    Ok(())
}

/// Install Forge or NeoForge loader
async fn install_forge(
    sender: Option<Arc<Sender<GenericProgress>>>,
    instance_selection: &InstanceSelection,
    version: Option<String>,
    is_neoforge: bool,
) -> Result<(), InstancePackageError> {
    let (f_send, f_recv) = std::sync::mpsc::channel();
    if let Some(sender) = sender.clone() {
        std::thread::spawn(move || {
            pipe_progress(f_recv, &sender);
        });
    }

    if is_neoforge {
        ql_mod_manager::loaders::neoforge::install(
            version,
            instance_selection.clone(),
            Some(f_send),
            None,
        )
        .await?;
    } else {
        ql_mod_manager::loaders::forge::install(
            version,
            instance_selection.clone(),
            Some(f_send),
            None,
        )
        .await?;
    }
    Ok(())
}

/// Copy files from MultiMC instance directory
async fn copy_multimc_files(
    temp_dir: &Path,
    sender: Option<Arc<Sender<GenericProgress>>>,
    instance_selection: &InstanceSelection,
) -> Result<(), InstancePackageError> {
    if let Some(sender) = sender.as_deref() {
        _ = sender.send(GenericProgress {
            done: 2,
            total: OUT_OF,
            message: Some("Copying files...".to_owned()),
            has_finished: false,
        });
    }

    // Copy minecraft folder (contains saves, mods, config, etc.)
    let minecraft_src = temp_dir.join("minecraft");
    if minecraft_src.is_dir() {
        let minecraft_dst = instance_selection.get_dot_minecraft_path();
        file_utils::copy_dir_recursive(&minecraft_src, &minecraft_dst).await?;
    }

    // Copy other folders
    copy_folder_if_exists(temp_dir, instance_selection, "jarmods").await?;
    copy_folder_if_exists(temp_dir, instance_selection, "patches").await?;
    copy_folder_if_exists(temp_dir, instance_selection, ".minecraft").await?;

    Ok(())
}

/// Helper to copy a folder if it exists
async fn copy_folder_if_exists(
    temp_dir: &Path,
    instance_selection: &InstanceSelection,
    folder_name: &str,
) -> Result<(), InstancePackageError> {
    let src = temp_dir.join(folder_name);
    if src.is_dir() {
        let dst = instance_selection.get_instance_path().join(folder_name);
        file_utils::copy_dir_recursive(&src, &dst).await?;
    }
    Ok(())
}

/// Import jar mods from MultiMC format
async fn import_jar_mods(
    temp_dir: &Path,
    instance_selection: &InstanceSelection,
) -> Result<(), InstancePackageError> {
    let jarmods_dir = temp_dir.join("jarmods");
    if !jarmods_dir.is_dir() {
        return Ok(());
    }

    // Read patches to find jar mod order
    let patches_dir = temp_dir.join("patches");
    let mut jar_mod_order: Vec<String> = Vec::new();

    if patches_dir.is_dir() {
        let mut entries = fs::read_dir(&patches_dir).await.path(&patches_dir)?;
        while let Some(entry) = entries.next_entry().await.path(&patches_dir)? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await.path(&path)?;
                if let Ok(patch) = serde_json::from_str::<MmcPatch>(&content) {
                    if let Some(jar_mods) = patch.jar_mods {
                        for jar_mod in jar_mods {
                            jar_mod_order.push(jar_mod.filename);
                        }
                    }
                }
            }
        }
    }

    // Create jar mods list for QuantumLauncher
    let mut jarmods = JarMods { mods: Vec::new() };

    // Add ordered jar mods first
    for filename in jar_mod_order {
        let src_path = jarmods_dir.join(&filename);
        if src_path.exists() {
            jarmods.mods.push(JarMod {
                filename: filename.clone(),
                enabled: true,
            });
        }
    }

    // Add any remaining jar mods
    let mut entries = fs::read_dir(&jarmods_dir).await.path(&jarmods_dir)?;
    while let Some(entry) = entries.next_entry().await.path(&jarmods_dir)? {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if !jarmods.mods.iter().any(|m| m.filename == filename) {
                    jarmods.mods.push(JarMod {
                        filename: filename.to_string(),
                        enabled: false,
                    });
                }
            }
        }
    }

    if !jarmods.mods.is_empty() {
        jarmods.save(instance_selection).await?;
    }

    Ok(())
}

/// Apply instance configuration from MultiMC
async fn apply_instance_config(
    cfg: &MmcInstanceCfg,
    instance_selection: &InstanceSelection,
) -> Result<(), InstancePackageError> {
    let mut config = InstanceConfigJson::read(instance_selection).await?;

    // Apply JVM arguments
    if let Some(jvm_args) = &cfg.jvm_args {
        let mut java_args = config.java_args.clone().unwrap_or_default();
        java_args.extend(jvm_args.split_whitespace().map(str::to_owned));
        config.java_args = Some(java_args);
    }

    // Apply Java path
    if let Some(java_path) = &cfg.java_path {
        config.java_override = Some(java_path.clone());
    }

    // Apply memory settings
    if let Some(max_mem) = cfg.max_memory {
        if let Some(args) = &mut config.java_args {
            args.retain(|arg| !arg.starts_with("-Xmx"));
            args.push(format!("-Xmx{}M", max_mem));
        } else {
            config.java_args = Some(vec![format!("-Xmx{}M", max_mem)]);
        }
    }

    if let Some(min_mem) = cfg.min_memory {
        if let Some(args) = &mut config.java_args {
            args.retain(|arg| !arg.starts_with("-Xms"));
            args.push(format!("-Xms{}M", min_mem));
        } else {
            let mut args = config.java_args.unwrap_or_default();
            args.push(format!("-Xms{}M", min_mem));
            config.java_args = Some(args);
        }
    }

    config.save(instance_selection).await?;
    Ok(())
}

async fn force_main_class_override(
    instance_selection: &InstanceSelection,
    class_name: &str,
) -> Result<(), InstancePackageError> {
    let mut config = InstanceConfigJson::read(instance_selection).await?;
    if config.main_class_override.as_deref() != Some(class_name) {
        config.main_class_override = Some(class_name.to_owned());
        config.save(instance_selection).await?;
        pt!(
            "Set main class override to '{}' for imported MultiMC instance",
            class_name
        );
    }
    Ok(())
}

/// Remove legacy-lwjgl3 library for jar mods that provide their own LWJGL implementation
async fn upgrade_legacy_lwjgl3(
    instance_selection: &InstanceSelection,
) -> Result<(), InstancePackageError> {
    use ql_core::json::version::VersionDetails;
    use ql_core::IntoJsonError;

    let details_path = instance_selection.get_instance_path().join("details.json");
    let contents = tokio::fs::read_to_string(&details_path)
        .await
        .path(&details_path)?;
    let mut details: VersionDetails = serde_json::from_str(&contents).json(contents)?;

    // Remove legacy-lwjgl3 library - jar mods like BTA include their own Display implementation
    // that would conflict with the standard legacy-lwjgl3 library
    details.libraries.retain(|library| {
        if let Some(name) = &library.name {
            if name.starts_with("org.mcphackers:legacy-lwjgl3:") {
                pt!("Removing legacy-lwjgl3 library - jar mod provides its own LWJGL implementation");
                return false;
            }
        }
        true
    });

    // Save updated version JSON
    let json_string = serde_json::to_string_pretty(&details).json(String::new())?;
    tokio::fs::write(&details_path, json_string)
        .await
        .path(&details_path)?;

    Ok(())
}

/// Import patches (additional component configurations)
async fn import_patches(
    temp_dir: &Path,
    _instance_selection: &InstanceSelection,
) -> Result<(), InstancePackageError> {
    let patches_dir = temp_dir.join("patches");
    if !patches_dir.is_dir() {
        return Ok(());
    }

    // Patches are already handled by jar mods import and file copying
    // Additional patch processing can be added here if needed
    Ok(())
}

/// Filter out problematic ByteArray entries from INI
fn filter_bytearray(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            !line.starts_with("mods_Page\\Columns")
                && !line.starts_with("ByteArray")
                && !line.starts_with("@ByteArray")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
