use std::sync::mpsc::Sender;
use std::fs::create_dir_all;
use std::fs::write;

use chrono::DateTime;
use chrono::Local;
use ql_core::{GenericProgress, InstanceSelection, Loader, do_jobs, info, json::VersionDetails};

use crate::store::{get_info_bulk, get_latest_version_date, toggle_mods};

use super::{ModError, ModId, ModIndex, delete_mods, download_mods_bulk};

pub async fn apply_updates(
    selected_instance: InstanceSelection,
    updates: Vec<ModId>,
    progress: Option<Sender<GenericProgress>>,
) -> Result<(), ModError> {
    let disabled_mods: Vec<_> = {
        let mod_index = ModIndex::load(&selected_instance).await?;
        updates
            .iter()
            .filter_map(|n| mod_index.mods.get_key_value(&n.get_index_str()))
            .filter(|n| !n.1.enabled)
            .map(|n| n.0.clone())
            .collect()
    };

    // It's as simple as that!
    delete_mods(updates.clone(), selected_instance.clone()).await?;
    download_mods_bulk(updates.clone(), selected_instance.clone(), progress).await?;

    write_changelog(updates, selected_instance.clone()).await;

    // Ensure disabled mods stay disabled
    toggle_mods(disabled_mods, selected_instance).await?;



    Ok(())
}


async fn write_changelog(mod_id: Vec<ModId>, selected_instance: InstanceSelection) {

    let titles = get_info_bulk(mod_id).await
            .unwrap()
            .into_iter()
            .map(|n|n.title)
            .collect::<Vec<_>>()
            .join("\n");

    let now = Local::now();
    let filename = format!("changelog-{}.txt", now.format("%Y-%m-%d-%H-%M"));
    let path = selected_instance.get_dot_minecraft_path().join("changelogs").join(&filename);

    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("Failed to create parent directories");
    }

    write(&path, &titles).expect("Failed to write file");
}

pub async fn check_for_updates(
    instance: InstanceSelection,
) -> Result<Vec<(ModId, String)>, ModError> {
    let index = ModIndex::load(&instance).await?;
    let version_json = VersionDetails::load(&instance).await?;

    let loader = instance.get_loader().await?;
    if let Loader::OptiFine = loader {
        return Ok(Vec::new());
    }
    info!(no_log, "Checking for mod updates (loader: {loader})");

    let version = version_json.get_id();

    let updated_mods: Result<Vec<Option<(ModId, String)>>, ModError> = do_jobs(
        index
            .mods
            .into_iter()
            .map(|(id, installed_mod)| async move {
                let mod_id = ModId::from_index_str(&id);

                let (download_version_time, download_version) =
                    get_latest_version_date(loader, &mod_id, version).await?;

                let installed_version_time =
                    DateTime::parse_from_rfc3339(&installed_mod.version_release_time)?;

                Ok((download_version_time > installed_version_time)
                    .then_some((mod_id, download_version)))
            }),
    )
    .await;
    let updated_mods: Vec<(ModId, String)> = updated_mods?.into_iter().flatten().collect();

    if updated_mods.is_empty() {
        info!(no_log, "No mod updates found");
    } else {
        info!(no_log, "Found mod updates");
    }

    Ok(updated_mods)
}
