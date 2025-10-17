//! MultiMC Instance Export
//!
//! Handles exporting QuantumLauncher instances to MultiMC/PrismLauncher format

use super::types::*;
use crate::InstancePackageError;
use ini::Ini;
use ql_core::{
    file_utils, info,
    jarmod::JarMods,
    json::{InstanceConfigJson, VersionDetails},
    pt, GenericProgress, InstanceSelection, IntoIoError, IntoJsonError, Loader,
};
use std::{collections::HashMap, path::Path, sync::mpsc::Sender};
use tokio::fs;

/// Export a QuantumLauncher instance to MultiMC/PrismLauncher format
pub async fn export_to_multimc(
    instance: InstanceSelection,
    progress: Option<Sender<GenericProgress>>,
) -> Result<Vec<u8>, InstancePackageError> {
    info!("Exporting instance to MultiMC format...");

    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 0,
            total: 5,
            message: Some("Preparing export...".to_owned()),
            has_finished: false,
        });
    }

    // Create temporary directory
    let temp_dir = tempfile::TempDir::new().map_err(InstancePackageError::TempDir)?;
    let export_path = temp_dir.path();

    // Read instance configuration
    let config = InstanceConfigJson::read(&instance).await?;
    let version_json = VersionDetails::load(&instance).await?;

    pt!("Exporting instance: {}", instance.get_name());
    pt!("Version: {}", version_json.get_id());

    // Create mmc-pack.json
    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 1,
            total: 5,
            message: Some("Creating mmc-pack.json...".to_owned()),
            has_finished: false,
        });
    }
    create_mmc_pack(&config, &version_json, export_path).await?;

    // Create instance.cfg
    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 2,
            total: 5,
            message: Some("Creating instance.cfg...".to_owned()),
            has_finished: false,
        });
    }
    create_instance_cfg(&instance, &config, export_path).await?;

    // Copy minecraft folder
    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 3,
            total: 5,
            message: Some("Copying game files...".to_owned()),
            has_finished: false,
        });
    }
    copy_minecraft_folder(&instance, export_path).await?;

    // Export jar mods if present
    export_jar_mods(&instance, export_path).await?;

    // Create pathmap.json
    create_pathmap(export_path).await?;

    // Zip the directory
    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 4,
            total: 5,
            message: Some("Compressing to zip...".to_owned()),
            has_finished: false,
        });
    }
    let bytes = file_utils::zip_directory_to_bytes(export_path)
        .await
        .map_err(InstancePackageError::ZipIo)?;

    if let Some(prog) = &progress {
        _ = prog.send(GenericProgress {
            done: 5,
            total: 5,
            message: Some("Export complete!".to_owned()),
            has_finished: true,
        });
    }

    info!("Finished exporting to MultiMC format");
    Ok(bytes)
}

/// Create mmc-pack.json
async fn create_mmc_pack(
    config: &InstanceConfigJson,
    version_json: &VersionDetails,
    export_path: &Path,
) -> Result<(), InstancePackageError> {
    let mut components = Vec::new();

    // Add Minecraft component
    components.push(MmcComponent {
        uid: "net.minecraft".to_string(),
        version: Some(version_json.id.clone()),
        cached_name: Some("Minecraft".to_string()),
        cached_version: Some(version_json.id.clone()),
        important: Some(true),
        dependency_only: None,
        cached_requires: None,
        cached_conflicts: None,
    });

    // Add LWJGL component if needed
    if version_json.id.ends_with("-lwjgl3") {
        components.push(MmcComponent {
            uid: "org.lwjgl3".to_string(),
            version: Some("3.3.3".to_string()),
            cached_name: Some("LWJGL 3".to_string()),
            cached_version: Some("3.3.3".to_string()),
            important: Some(false),
            dependency_only: Some(true),
            cached_requires: None,
            cached_conflicts: None,
        });
    } else {
        components.push(MmcComponent {
            uid: "org.lwjgl".to_string(),
            version: Some("2.9.4".to_string()),
            cached_name: Some("LWJGL 2".to_string()),
            cached_version: Some("2.9.4".to_string()),
            important: Some(false),
            dependency_only: Some(true),
            cached_requires: None,
            cached_conflicts: None,
        });
    }

    // Add loader component
    if let Ok(loader) = Loader::try_from(config.mod_type.as_str()) {
        match loader {
            Loader::Forge => {
                if let Some(forge_version) = get_forge_version(config).await {
                    components.push(MmcComponent {
                        uid: "net.minecraftforge".to_string(),
                        version: Some(forge_version.clone()),
                        cached_name: Some("Forge".to_string()),
                        cached_version: Some(forge_version),
                        important: Some(false),
                        dependency_only: None,
                        cached_requires: None,
                        cached_conflicts: None,
                    });
                }
            }
            Loader::Neoforge => {
                if let Some(neoforge_version) = get_forge_version(config).await {
                    components.push(MmcComponent {
                        uid: "net.neoforged".to_string(),
                        version: Some(neoforge_version.clone()),
                        cached_name: Some("NeoForge".to_string()),
                        cached_version: Some(neoforge_version),
                        important: Some(false),
                        dependency_only: None,
                        cached_requires: None,
                        cached_conflicts: None,
                    });
                }
            }
            Loader::Fabric => {
                if let Some(fabric_version) = get_fabric_version(config).await {
                    components.push(MmcComponent {
                        uid: "net.fabricmc.fabric-loader".to_string(),
                        version: Some(fabric_version.clone()),
                        cached_name: Some("Fabric Loader".to_string()),
                        cached_version: Some(fabric_version),
                        important: Some(false),
                        dependency_only: None,
                        cached_requires: None,
                        cached_conflicts: None,
                    });
                }
            }
            Loader::Quilt => {
                if let Some(quilt_version) = get_fabric_version(config).await {
                    components.push(MmcComponent {
                        uid: "org.quiltmc.quilt-loader".to_string(),
                        version: Some(quilt_version.clone()),
                        cached_name: Some("Quilt Loader".to_string()),
                        cached_version: Some(quilt_version),
                        important: Some(false),
                        dependency_only: None,
                        cached_requires: None,
                        cached_conflicts: None,
                    });
                }
            }
            _ => {}
        }
    }

    let mmc_pack = MmcPack {
        format_version: 1,
        components,
    };

    let json = serde_json::to_string_pretty(&mmc_pack).json_to()?;
    let pack_path = export_path.join("mmc-pack.json");
    fs::write(&pack_path, json).await.path(&pack_path)?;

    Ok(())
}

/// Get Forge/NeoForge version from config
async fn get_forge_version(config: &InstanceConfigJson) -> Option<String> {
    // Try to extract version from forge/neoforge installation
    // This is a simplified version - you may need to parse forge JSON files
    config
        .main_class_override
        .as_ref()
        .and_then(|_| Some("43.2.8".to_string())) // Placeholder
}

/// Get Fabric/Quilt version from config
async fn get_fabric_version(config: &InstanceConfigJson) -> Option<String> {
    // Try to extract version from fabric.json or config
    config
        .main_class_override
        .as_ref()
        .and_then(|_| Some("0.15.11".to_string())) // Placeholder
}

/// Create instance.cfg
async fn create_instance_cfg(
    instance: &InstanceSelection,
    config: &InstanceConfigJson,
    export_path: &Path,
) -> Result<(), InstancePackageError> {
    let mut ini = Ini::new();

    // General section
    ini.with_section(Some("General"))
        .set("InstanceType", "OneSix")
        .set("name", instance.get_name())
        .set("iconKey", "default");

    // Add JVM args
    if let Some(java_args) = &config.java_args {
        let args = java_args.join(" ");
        ini.with_section(Some("General")).set("JvmArgs", &args);
    }

    // Add Java path
    if let Some(java_path) = &config.java_override {
        ini.with_section(Some("General"))
            .set("JavaPath", java_path)
            .set("OverrideJavaLocation", "true");
    }

    // Extract memory settings from JVM args
    if let Some(java_args) = &config.java_args {
        for arg in java_args {
            if arg.starts_with("-Xmx") {
                if let Some(mem) = arg.strip_prefix("-Xmx").and_then(|s| s.strip_suffix("M")) {
                    ini.with_section(Some("General"))
                        .set("MaxMemAlloc", mem)
                        .set("OverrideMemory", "true");
                }
            }
            if arg.starts_with("-Xms") {
                if let Some(mem) = arg.strip_prefix("-Xms").and_then(|s| s.strip_suffix("M")) {
                    ini.with_section(Some("General"))
                        .set("MinMemAlloc", mem)
                        .set("OverrideMemory", "true");
                }
            }
        }
    }

    // Write INI file
    let cfg_path = export_path.join("instance.cfg");
    ini.write_to_file(&cfg_path).path(&cfg_path)?;

    Ok(())
}

/// Copy minecraft folder
async fn copy_minecraft_folder(
    instance: &InstanceSelection,
    export_path: &Path,
) -> Result<(), InstancePackageError> {
    let minecraft_src = instance.get_dot_minecraft_path();
    let minecraft_dst = export_path.join("minecraft");

    if minecraft_src.is_dir() {
        // Exclude certain folders that shouldn't be exported
        let excludes = vec![
            minecraft_src.join("versions"),
            minecraft_src.join("libraries"),
            minecraft_src.join("usercache.json"),
            minecraft_src.join("launcher_profiles.json"),
        ];

        file_utils::copy_dir_recursive_ext(&minecraft_src, &minecraft_dst, &excludes).await?;
    }

    Ok(())
}

/// Export jar mods
async fn export_jar_mods(
    instance: &InstanceSelection,
    export_path: &Path,
) -> Result<(), InstancePackageError> {
    let jarmods_src = instance.get_instance_path().join("jarmods");
    if !jarmods_src.is_dir() {
        return Ok(());
    }

    let jarmods_dst = export_path.join("jarmods");
    fs::create_dir_all(&jarmods_dst).await.path(&jarmods_dst)?;

    // Copy jar mod files
    file_utils::copy_dir_recursive(&jarmods_src, &jarmods_dst).await?;

    // Create patches directory
    let patches_dst = export_path.join("patches");
    fs::create_dir_all(&patches_dst).await.path(&patches_dst)?;

    // Read jar mods configuration
    if let Ok(jarmods_json) = JarMods::get(instance).await {
        if !jarmods_json.mods.is_empty() {
            // Create a patch file for jar mods
            let mut jar_mod_entries = Vec::new();
            for jar_mod in &jarmods_json.mods {
                if jar_mod.enabled {
                    jar_mod_entries.push(MmcJarMod {
                        display_name: jar_mod
                            .filename
                            .strip_suffix(".zip")
                            .or_else(|| jar_mod.filename.strip_suffix(".jar"))
                            .unwrap_or(&jar_mod.filename)
                            .to_string(),
                        filename: jar_mod.filename.clone(),
                    });
                }
            }

            if !jar_mod_entries.is_empty() {
                let patch = MmcPatch {
                    format_version: Some(1),
                    uid: Some("custom.jarmods".to_string()),
                    name: Some("Jar Mods".to_string()),
                    version: Some("1.0".to_string()),
                    main_class: None,
                    minecraft_arguments: None,
                    libraries: None,
                    jar_mods: Some(jar_mod_entries),
                    asset_index: None,
                    main_jar: None,
                    traits: Some(vec!["jarmod".to_string()]),
                    order: Some(100),
                };

                let patch_json = serde_json::to_string_pretty(&patch).json_to()?;
                let patch_path = patches_dst.join("custom.jarmods.json");
                fs::write(&patch_path, patch_json).await.path(&patch_path)?;
            }
        }
    }

    Ok(())
}

/// Create pathmap.json for portable file mapping
async fn create_pathmap(export_path: &Path) -> Result<(), InstancePackageError> {
    let mut mappings = HashMap::new();

    // Map common paths
    mappings.insert("minecraft".to_string(), "/minecraft".to_string());
    mappings.insert("jarmods".to_string(), "/jarmods".to_string());
    mappings.insert("patches".to_string(), "/patches".to_string());
    mappings.insert("instance.cfg".to_string(), "/instance.cfg".to_string());
    mappings.insert("mmc-pack.json".to_string(), "/mmc-pack.json".to_string());

    // Scan jarmods directory if exists
    let jarmods_dir = export_path.join("jarmods");
    if jarmods_dir.is_dir() {
        let mut entries = fs::read_dir(&jarmods_dir).await.path(&jarmods_dir)?;
        while let Some(entry) = entries.next_entry().await.path(&jarmods_dir)? {
            if let Some(filename) = entry.file_name().to_str() {
                mappings.insert(
                    format!("jarmods/{}", filename),
                    format!("/jarmods/{}", filename),
                );
            }
        }
    }

    // Scan patches directory if exists
    let patches_dir = export_path.join("patches");
    if patches_dir.is_dir() {
        let mut entries = fs::read_dir(&patches_dir).await.path(&patches_dir)?;
        while let Some(entry) = entries.next_entry().await.path(&patches_dir)? {
            if let Some(filename) = entry.file_name().to_str() {
                mappings.insert(
                    format!("patches/{}", filename),
                    format!("/patches/{}", filename),
                );
            }
        }
    }

    let pathmap = MmcPathMap { mappings };
    let json = serde_json::to_string_pretty(&pathmap).json_to()?;
    let pathmap_path = export_path.join("pathmap.json");
    fs::write(&pathmap_path, json).await.path(&pathmap_path)?;

    Ok(())
}
