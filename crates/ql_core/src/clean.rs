use std::{collections::HashSet, fs::Metadata, path::Path};

use fs::DirEntry;
use tokio::fs;

use crate::{
    file_utils::get_launcher_dir,
    info,
    json::{AssetIndex, VersionDetails},
    pt, IntoIoError, IntoJsonError, IoError, JsonFileError, LAUNCHER_DIR,
};

pub async fn dir(path: &str) -> Result<(), IoError> {
    const SIZE_LIMIT_BYTES: u64 = 100 * 1024 * 1024; // 100 MB

    let dir = get_launcher_dir()?.join(path);
    if !dir.is_dir() {
        fs::create_dir_all(&dir).await.path(dir)?;
        return Ok(());
    }
    let mut total_size = 0;
    let mut files: Vec<(DirEntry, Metadata)> = Vec::new();

    let mut read_dir = fs::read_dir(&dir).await.dir(&dir)?;

    while let Some(entry) = read_dir.next_entry().await.dir(&dir)? {
        let metadata = entry.metadata().await.path(entry.path())?;
        if metadata.is_file() {
            total_size += metadata.len();
            files.push((entry, metadata));
        }
    }

    if total_size <= SIZE_LIMIT_BYTES {
        return Ok(());
    }

    info!("Cleaning up {dir:?}");
    files.sort_unstable_by_key(|(_, metadata)| {
        metadata.modified().unwrap_or(std::time::SystemTime::now())
    });

    let mut cleaned_amount = 0;

    for (file, metadata) in files {
        let path = file.path();
        fs::remove_file(&path).await.path(path)?;
        let len = metadata.len();
        total_size -= len;
        cleaned_amount += len;

        if total_size <= SIZE_LIMIT_BYTES {
            break;
        }
    }

    pt!(
        "Cleaned {:.1} MB",
        cleaned_amount as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}

pub async fn assets_dir() -> Result<u64, JsonFileError> {
    let assets_dir = LAUNCHER_DIR.join("assets/dir");
    let indexes_dir = assets_dir.join("indexes");

    let indexes = get_used_indexes().await?;
    let hashes = get_used_hashes(&indexes_dir, &indexes).await?;

    let mut cleaned_size = 0;

    let objects_dir = assets_dir.join("objects");
    let mut objects = fs::read_dir(&objects_dir).await.path(&objects_dir)?;
    while let Some(next) = objects.next_entry().await.path(&objects_dir)? {
        let object_dir_path = next.path();
        let mut object_dir = fs::read_dir(&object_dir_path)
            .await
            .path(&object_dir_path)?;

        let mut dir_is_empty = true;
        while let Some(object) = object_dir.next_entry().await.path(&object_dir_path)? {
            let name = object.file_name().to_string_lossy().to_string();
            if hashes.contains(&name) {
                dir_is_empty = false;
            } else {
                let path = object.path();
                let metadata = object.metadata().await.path(&path)?;
                cleaned_size += metadata.len();

                fs::remove_file(&path).await.path(path)?;
            }
        }

        if dir_is_empty {
            fs::remove_dir_all(&object_dir_path)
                .await
                .path(&object_dir_path)?;
        }
    }

    Ok(cleaned_size)
}

async fn get_used_hashes(
    indexes_dir: &Path,
    index_files: &[String],
) -> Result<HashSet<String>, JsonFileError> {
    let mut jsons = Vec::new();

    let mut indexes = fs::read_dir(&indexes_dir).await.path(&indexes_dir)?;
    while let Some(next) = indexes.next_entry().await.path(&indexes_dir)? {
        let path = next.path();
        let name = next.file_name();
        if !index_files.iter().any(|n| **n == name) {
            fs::remove_file(&path).await.path(path)?;
            continue;
        }

        let json = fs::read_to_string(&path).await.path(path)?;
        let json: AssetIndex = serde_json::from_str(&json).json(json)?;
        jsons.push(json);
    }

    let hashes: HashSet<String> = jsons
        .into_iter()
        .map(|n| n.objects.into_values().map(|n| n.hash))
        .flatten()
        .collect();

    Ok(hashes)
}

async fn get_used_indexes() -> Result<Vec<String>, JsonFileError> {
    let instances_dir = LAUNCHER_DIR.join("instances");
    let mut instances = fs::read_dir(&instances_dir).await.path(&instances_dir)?;

    let mut used_files = Vec::new();

    while let Some(instance) = instances.next_entry().await.path(&instances_dir)? {
        let json_path = instance.path().join("details.json");
        if !fs::try_exists(&json_path).await.path(&json_path)? {
            continue;
        }
        let json = fs::read_to_string(&json_path).await.path(&json_path)?;
        let json: VersionDetails = serde_json::from_str(&json).json(json)?;
        used_files.push(format!("{}.json", json.assetIndex.id));
    }

    Ok(used_files)
}
