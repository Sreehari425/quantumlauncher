use std::sync::mpsc::Sender;

use ql_core::{
    info,
    json::version::VersionDetails,
    DownloadProgress, GenericProgress, InstanceSelection, IntoIoError, ListEntry, LAUNCHER_DIR,
    LAUNCHER_VERSION_NAME,
};

mod downloader;
mod libraries;

pub use downloader::DownloadError;
pub(crate) use downloader::GameDownloader;

/// Creates a Minecraft instance.
///
/// # Arguments
/// - `instance_name` : Name of the instance (for example: "my cool instance")
/// - `version` : Version of the game to download (for example: "1.21.1", "1.12.2", "b1.7.3", etc.)
/// - `progress_sender` : If you want, you can create an `mpsc::channel()` of [`DownloadProgress`],
///   provide the receiver and keep polling the sender for progress updates. *If not needed, leave as `None`*
/// - `download_assets` : Whether to download the assets. Default: true. Disable this if you want to speed
///   up the download or reduce file size. *Disabling this will make the game completely silent;
///   No sounds or music will play*
///
/// # Returns
/// The instance name that you passed in.
///
/// # Errors
/// Check the [`DownloadError`] documentation (if there is, lol).
/// This is crap code and you must have standards. (WTF: )
pub async fn create_instance(
    instance_name: String,
    version: ListEntry,
    progress_sender: Option<Sender<DownloadProgress>>,
    download_assets: bool,
) -> Result<String, DownloadError> {
    info!("Started creating instance.");
    debug_assert!(!version.is_server);

    // An empty asset directory.
    let launcher_dir = &*LAUNCHER_DIR;

    let assets_dir = launcher_dir.join("assets/null");
    tokio::fs::create_dir_all(&assets_dir)
        .await
        .path(assets_dir)?;

    let mut game_downloader =
        GameDownloader::new(&instance_name, &version, progress_sender).await?;

    game_downloader.download_logging_config().await?;
    game_downloader.download_jar().await?;
    game_downloader.download_libraries().await?;
    game_downloader.library_extras().await?;

    if download_assets {
        game_downloader.download_assets().await?;
    }

    game_downloader
        .version_json
        .save_to_dir(&game_downloader.instance_dir)
        .await?;
    game_downloader.create_profiles_json().await?;
    game_downloader.create_config_json().await?;

    let version_file_path = launcher_dir
        .join("instances")
        .join(&instance_name)
        .join("launcher_version.txt");
    tokio::fs::write(&version_file_path, LAUNCHER_VERSION_NAME)
        .await
        .path(version_file_path)?;

    let mods_dir = launcher_dir
        .join("instances")
        .join(&instance_name)
        .join(".minecraft/mods");
    tokio::fs::create_dir_all(&mods_dir).await.path(mods_dir)?;

    info!("Finished creating instance: {instance_name}");

    Ok(instance_name)
}

/// Redownloads native libraries for an existing instance.
///
/// This function deletes the existing natives folder and re-downloads
/// all native libraries from scratch. Useful when natives are corrupted
/// or when switching platforms.
///
/// # Arguments
/// - `instance` : The instance selection (client or server)
/// - `progress_sender` : Optional sender for progress updates
///
/// # Errors
/// Returns a [`DownloadError`] if the redownload fails.
pub async fn redownload_natives(
    instance: &InstanceSelection,
    progress_sender: Option<Sender<GenericProgress>>,
) -> Result<(), DownloadError> {
    info!("Starting redownload of natives for instance: {}", instance.get_name());

    let instance_dir = instance.get_instance_path();
    let version_json = VersionDetails::load(instance).await?;

    // Delete existing natives folder
    let natives_path = instance_dir.join("libraries/natives");
    if natives_path.exists() {
        info!("Removing existing natives folder");
        tokio::fs::remove_dir_all(&natives_path)
            .await
            .path(&natives_path)?;
    }

    // Create fresh natives directory
    tokio::fs::create_dir_all(&natives_path)
        .await
        .path(&natives_path)?;

    // Create a game downloader with the existing instance
    let game_downloader = GameDownloader::with_existing_instance(
        version_json,
        instance_dir.clone(),
        None, // We'll use GenericProgress instead
    );

    // Count total libraries for progress
    let total_libs = game_downloader.version_json.libraries.len();
    let mut current = 0;

    // Re-download libraries (this will re-extract natives)
    for library in &game_downloader.version_json.libraries {
        if !library.is_allowed() {
            continue;
        }

        game_downloader.download_library(library, None).await?;

        current += 1;
        if let Some(sender) = &progress_sender {
            let _ = sender.send(GenericProgress {
                done: current,
                total: total_libs,
                message: Some(format!("Redownloading library {current}/{total_libs}")),
                has_finished: false,
            });
        }
    }

    // Run library extras (e.g., FreeBSD LWJGL2 natives)
    game_downloader.library_extras().await?;

    if let Some(sender) = &progress_sender {
        let _ = sender.send(GenericProgress {
            done: total_libs,
            total: total_libs,
            message: Some("Finished redownloading natives".to_string()),
            has_finished: true,
        });
    }

    info!("Finished redownloading natives for instance: {}", instance.get_name());

    Ok(())
}
