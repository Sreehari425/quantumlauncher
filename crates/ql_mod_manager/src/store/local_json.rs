use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
};

use ql_core::{info, InstanceSelection, IntoIoError, IntoJsonError, JsonFileError};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::ModError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModConfig {
    pub name: String,
    pub manually_installed: bool,
    pub installed_version: String,
    pub version_release_time: String,
    pub enabled: bool,
    pub description: String,
    pub icon_url: Option<String>,
    /// Source platform where the mod was downloaded from.
    /// Eg: "modrinth"
    pub project_source: String,
    pub project_id: String,
    pub files: Vec<ModFile>,
    pub supported_versions: Vec<String>,
    pub dependencies: HashSet<String>,
    pub dependents: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModIndex {
    pub mods: HashMap<String, ModConfig>,
    pub is_server: Option<bool>,
}

impl ModIndex {
    pub async fn load(selected_instance: &InstanceSelection) -> Result<Self, JsonFileError> {
        let dot_mc_dir = selected_instance.get_dot_minecraft_path();

        let mods_dir = dot_mc_dir.join("mods");
        if !mods_dir.exists() {
            fs::create_dir(&mods_dir).await.path(&mods_dir)?;
        }

        let index_path = dot_mc_dir.join("mod_index.json");
        let old_index_path = mods_dir.join("index.json");

        // 1) Try migrating old index
        match fs::read_to_string(&old_index_path).await {
            Ok(index) => {
                let mod_index = serde_json::from_str(&index).json(index.clone())?;

                fs::write(&index_path, &index).await.path(index_path)?;
                fs::remove_file(&old_index_path)
                    .await
                    .path(old_index_path)?;

                return Ok(mod_index);
            }
            Err(e) if e.kind() != ErrorKind::NotFound => {
                return Err(e.path(index_path).into());
            }
            _ => {}
        }

        // 2. Try current index
        match fs::read_to_string(&index_path).await {
            Ok(index) => return Ok(serde_json::from_str::<Self>(&index).json(index)?),
            Err(e) if e.kind() != ErrorKind::NotFound => {
                return Err(e.path(index_path).into());
            }
            _ => {}
        }

        let index = ModIndex::new(selected_instance);
        let index_str = serde_json::to_string(&index).json_to()?;

        let tmp = index_path.with_extension("json.tmp");
        fs::write(&tmp, &index_str).await.path(&tmp)?;
        fs::rename(&tmp, &index_path).await.path(&tmp)?;

        Ok(index)
    }

    pub async fn save(&mut self, selected_instance: &InstanceSelection) -> Result<(), ModError> {
        self.fix(selected_instance).await?;

        let index_dir = selected_instance
            .get_dot_minecraft_path()
            .join("mod_index.json");

        let index_str = serde_json::to_string(&self).json_to()?;
        fs::write(&index_dir, &index_str).await.path(index_dir)?;
        Ok(())
    }

    fn new(instance_name: &InstanceSelection) -> Self {
        Self {
            mods: HashMap::new(),
            is_server: Some(instance_name.is_server()),
        }
    }

    pub async fn fix(&mut self, selected_instance: &InstanceSelection) -> Result<(), ModError> {
        let mods_dir = selected_instance.get_dot_minecraft_path().join("mods");
        if !mods_dir.exists() {
            fs::create_dir(&mods_dir).await.path(&mods_dir)?;
            return Ok(());
        }

        let mut removed_ids = Vec::new();
        let mut remove_dependents = Vec::new();

        for (id, mod_cfg) in &mut self.mods {
            mod_cfg.files.retain(|file| {
                mods_dir.join(&file.filename).is_file()
                    || mods_dir
                        .join(format!("{}.disabled", file.filename))
                        .is_file()
            });
            if mod_cfg.files.is_empty() {
                info!("Cleaning deleted mod: {}", mod_cfg.name);
                removed_ids.push(id.clone());
            }
            for dependent in &mod_cfg.dependents {
                remove_dependents.push((dependent.clone(), id.clone()));
            }
        }

        for (dependent, dependency) in remove_dependents {
            if let Some(mod_cfg) = self.mods.get_mut(&dependent) {
                mod_cfg.dependencies.remove(&dependency);
            }
        }

        for id in removed_ids {
            self.mods.remove(&id);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModFile {
    // pub hashes: ModHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    // pub size: usize,
    // pub file_type: Option<String>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ModHashes {
//     pub sha512: String,
//     pub sha1: String,
// }
