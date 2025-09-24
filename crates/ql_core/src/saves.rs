//! This module provides functionality to read and manage Minecraft save files

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::{InstanceSelection, IntoIoError, IoError};

/// Represents a Minecraft save/world
#[derive(Debug, Clone)]
pub struct Save {
    /// The name of the save (folder name)
    pub name: String,
    /// Path to the save directory
    pub path: PathBuf,
    /// Path to the world icon (icon.png), if it exists
    pub icon_path: Option<PathBuf>,
    /// Whether the save has a level.dat file (indicates valid world)
    pub has_level_dat: bool,
    /// Size of the save directory in bytes (if calculable)
    pub size_bytes: Option<u64>,
}

impl Save {
    /// Creates a new Save instance with basic information
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            icon_path: None,
            has_level_dat: false,
            size_bytes: None,
        }
    }

    /// Checks if the save directory contains an icon.png file
    pub async fn check_icon(&mut self) -> Result<(), IoError> {
        let icon_path = self.path.join("icon.png");
        if icon_path.exists() {
            self.icon_path = Some(icon_path);
        }
        Ok(())
    }

    /// Checks for the presence of level.dat file
    pub async fn check_level_files(&mut self) -> Result<(), IoError> {
        let level_dat = self.path.join("level.dat");

        self.has_level_dat = level_dat.exists();

        Ok(())
    }

    /// Calculates the total size of the save directory
    pub async fn calculate_size(&mut self) -> Result<(), IoError> {
        self.size_bytes = Some(calculate_directory_size(&self.path).await?);
        Ok(())
    }
}

/// Reads all save information from a Minecraft instance
///
/// Returns a vector of Save structs containing metadata about each world.
///
/// # Arguments
/// * `instance` - The instance selection to read saves from
///
/// # Returns
/// A Vec of Save structs, or an IoError if the saves directory cannot be read
///
/// # Example
/// ```no_run
/// use ql_core::{InstanceSelection, saves::read_saves_info};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let instance = InstanceSelection::new("MyInstance", false);
/// let saves = read_saves_info(&instance).await?;
///
/// for save in saves {
///     println!("Found save: {} at {:?}", save.name, save.path);
///     if let Some(icon) = save.icon_path {
///         println!("  Has icon: {:?}", icon);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn read_saves_info(instance: &InstanceSelection) -> Result<Vec<Save>, IoError> {
    let saves_dir = get_saves_directory(instance);

    // Check if saves directory exists
    if !saves_dir.exists() {
        return Ok(Vec::new());
    }

    let mut saves = Vec::new();

    // Read directory contents
    let mut entries = fs::read_dir(&saves_dir).await.path(&saves_dir)?;

    while let Some(entry) = entries.next_entry().await.path(&saves_dir)? {
        let entry_path = entry.path();

        // Only process directories
        if entry_path.is_dir() {
            let save_name = entry.file_name().to_string_lossy().to_string();

            let mut save = Save::new(save_name.clone(), entry_path);

            if let Err(_) = save.check_icon().await {
                // Continue even if icon check fails :)
            }

            if let Err(_) = save.check_level_files().await {
                // Continue even if level file check fails :)
            }

            // Only include saves that have at least a level.dat file (valid worlds)
            if save.has_level_dat {
                saves.push(save);
            }
        }
    }

    Ok(saves)
}

/// Gets the saves directory path for a given instance
///
/// For client instances, this is `.minecraft/saves`
/// For servers, this is the server root directory (servers don't typically have multiple worlds)
fn get_saves_directory(instance: &InstanceSelection) -> PathBuf {
    match instance {
        InstanceSelection::Instance(_) => {
            // Client instance saves are in .minecraft/saves
            instance.get_dot_minecraft_path().join("saves")
        }
        InstanceSelection::Server(_) => {
            // Server saves are typically in the server root/world
            instance.get_instance_path().join("world")
        }
    }
}

/// Recursively calculates the size of a directory
async fn calculate_directory_size(dir: &Path) -> Result<u64, IoError> {
    let mut total_size = 0u64;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = match fs::read_dir(&current_dir).await {
            Ok(entries) => entries,
            Err(_) => continue, // Skip directories we can't read
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                if let Ok(metadata) = entry.metadata().await {
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(total_size)
}
