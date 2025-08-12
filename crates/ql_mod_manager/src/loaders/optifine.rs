use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::Sender,
};

use ql_core::{
    file_utils, impl_3_errs_jri, info, jarmod,
    json::{optifine::JsonOptifine, VersionDetails},
    no_window, pt, GenericProgress, InstanceSelection, IntoIoError, IntoJsonError, IoError,
    JsonError, Progress, RequestError, CLASSPATH_SEPARATOR, LAUNCHER_DIR,
};
use ql_java_handler::{get_java_binary, JavaInstallError, JavaVersion, JAVA};
use thiserror::Error;

use super::change_instance_type;

pub async fn install_b173(
    instance: InstanceSelection,
    url: &'static str,
) -> Result<(), OptifineError> {
    let bytes = file_utils::download_file_to_bytes(url, true).await?;
    jarmod::insert(instance, bytes, "Optifine").await?;

    Ok(())
}

// javac -cp OptiFine_1.21.1_HD_U_J1.jar OptifineInstaller.java -d .
// java -cp OptiFine_1.21.1_HD_U_J1.jar:. OptifineInstaller

#[derive(Default)]
pub enum OptifineInstallProgress {
    #[default]
    P1Start,
    P2CompilingHook,
    P3RunningHook,
    P4DownloadingLibraries {
        done: usize,
        total: usize,
    },
    P5Done,
}

impl Display for OptifineInstallProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptifineInstallProgress::P1Start => write!(f, "Starting installation."),
            OptifineInstallProgress::P2CompilingHook => write!(f, "Compiling hook."),
            OptifineInstallProgress::P3RunningHook => write!(f, "Running hook."),
            OptifineInstallProgress::P4DownloadingLibraries { done, total } => {
                write!(f, "Downloading libraries ({done}/{total}).")
            }
            OptifineInstallProgress::P5Done => write!(f, "Done."),
        }
    }
}

impl Progress for OptifineInstallProgress {
    fn get_num(&self) -> f32 {
        match self {
            OptifineInstallProgress::P1Start => 0.0,
            OptifineInstallProgress::P2CompilingHook => 1.0,
            OptifineInstallProgress::P3RunningHook => 2.0,
            OptifineInstallProgress::P4DownloadingLibraries { done, total } => {
                2.0 + (*done as f32 / *total as f32)
            }
            OptifineInstallProgress::P5Done => 3.0,
        }
    }

    fn get_message(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn total() -> f32 {
        3.0
    }
}

pub async fn install(
    instance_name: String,
    path_to_installer: PathBuf,
    progress_sender: Option<Sender<OptifineInstallProgress>>,
    java_progress_sender: Option<Sender<GenericProgress>>,
    optifine_unique_version: bool,
) -> Result<(), OptifineError> {
    if !path_to_installer.exists() || !path_to_installer.is_file() {
        return Err(OptifineError::InstallerDoesNotExist);
    }

    let progress_sender = progress_sender.as_ref();

    info!("Started installing OptiFine");
    send_progress(progress_sender, OptifineInstallProgress::P1Start);

    if optifine_unique_version {
        let installer = tokio::fs::read(&path_to_installer)
            .await
            .path(&path_to_installer)?;
        jarmod::insert(
            InstanceSelection::Instance(instance_name),
            installer,
            "Optifine",
        )
        .await?;
        pt!("Finished installing OptiFine for Beta 1.7.3");
        return Ok(());
    }

    let instance_path = LAUNCHER_DIR.join("instances").join(&instance_name);
    create_details_json(&instance_path).await?;
    let dot_minecraft_path = instance_path.join(".minecraft");

    let optifine_path = instance_path.join("optifine");
    tokio::fs::create_dir_all(&optifine_path)
        .await
        .path(&optifine_path)?;

    create_hook_java_file(&dot_minecraft_path, &optifine_path).await?;

    let new_installer_path = optifine_path.join("OptiFine.jar");
    tokio::fs::copy(&path_to_installer, &new_installer_path)
        .await
        .path(path_to_installer)?;

    pt!("Compiling OptifineInstaller.java");
    send_progress(progress_sender, OptifineInstallProgress::P2CompilingHook);
    compile_hook(
        &new_installer_path,
        &optifine_path,
        java_progress_sender.as_ref(),
    )
    .await?;

    pt!("Running OptifineInstaller.java");
    send_progress(progress_sender, OptifineInstallProgress::P3RunningHook);
    run_hook(&new_installer_path, &optifine_path).await?;

    download_libraries(&instance_name, &dot_minecraft_path, progress_sender).await?;
    change_instance_type(&instance_path, "OptiFine".to_owned()).await?;
    send_progress(progress_sender, OptifineInstallProgress::P5Done);
    pt!("Finished installing OptiFine");

    Ok(())
}

fn send_progress(
    progress_sender: Option<&Sender<OptifineInstallProgress>>,
    prog: OptifineInstallProgress,
) {
    if let Some(progress) = progress_sender {
        _ = progress.send(prog);
    }
}

pub async fn uninstall(instance_name: String) -> Result<(), OptifineError> {
    let instance_path = LAUNCHER_DIR.join("instances").join(&instance_name);

    let optifine_path = instance_path.join("optifine");
    if optifine_path.is_dir() {
        tokio::fs::remove_dir_all(&optifine_path)
            .await
            .path(optifine_path)?;
    }

    change_instance_type(&instance_path, "Vanilla".to_owned()).await?;

    let dot_minecraft_path = instance_path.join(".minecraft");
    let libraries_path = dot_minecraft_path.join("libraries");
    if libraries_path.is_dir() {
        tokio::fs::remove_dir_all(&libraries_path)
            .await
            .path(libraries_path)?;
    }

    let versions_path = dot_minecraft_path.join("versions");
    if versions_path.is_dir() {
        let mut to_be_removed = Vec::new();
        file_utils::find_item_in_dir(&versions_path, |path, name| {
            if name.to_lowercase().contains("Opti") {
                to_be_removed.push(path.to_owned());
            }
            false
        })
        .await?;

        for rem in to_be_removed {
            tokio::fs::remove_dir_all(&rem).await.path(rem)?;
        }
    }
    Ok(())
}

async fn create_hook_java_file(
    dot_minecraft_path: &Path,
    optifine_path: &Path,
) -> Result<(), OptifineError> {
    let mc_path = dot_minecraft_path.to_str().unwrap().replace('\\', "\\\\");
    let hook = include_str!("../../../../assets/installers/OptifineInstaller.java")
        .replace("REPLACE_WITH_MC_PATH", &mc_path);
    let hook_path = optifine_path.join("OptifineInstaller.java");
    tokio::fs::write(&hook_path, hook).await.path(hook_path)?;
    Ok(())
}

async fn download_libraries(
    instance_name: &str,
    dot_minecraft_path: &Path,
    progress_sender: Option<&Sender<OptifineInstallProgress>>,
) -> Result<(), OptifineError> {
    let (optifine_json, _) = JsonOptifine::read(instance_name).await?;
    let libraries_path = dot_minecraft_path.join("libraries");

    let len = optifine_json.libraries.len();
    for (i, library) in optifine_json
        .libraries
        .iter()
        .filter_map(|lib| (!lib.name.starts_with("optifine")).then_some(&lib.name))
        .enumerate()
    {
        // l = com.mojang:netty:1.8.8
        // path = com/mojang/netty/1.8.8/netty-1.8.8.jar
        // url = https://libraries.minecraft.net/com/mojang/netty/1.8.8/netty-1.8.8.jar

        // Split in colon
        let parts: Vec<&str> = library.split(':').collect();

        info!("Downloading library ({i}/{len}): {}", library);

        let url_parent_path = format!("{}/{}/{}", parts[0].replace('.', "/"), parts[1], parts[2],);
        let url_final_part = format!("{url_parent_path}/{}-{}.jar", parts[1], parts[2],);

        let parent_path = libraries_path.join(&url_parent_path);
        tokio::fs::create_dir_all(&parent_path)
            .await
            .path(parent_path)?;

        let url = format!("https://libraries.minecraft.net/{url_final_part}");

        let jar_path = libraries_path.join(&url_final_part);

        if let Some(progress) = progress_sender {
            _ = progress.send(OptifineInstallProgress::P4DownloadingLibraries {
                done: i,
                total: len,
            });
        }

        if jar_path.exists() {
            continue;
        }
        file_utils::download_file_to_path(&url, false, &jar_path).await?;
    }
    Ok(())
}

async fn run_hook(new_installer_path: &Path, optifine_path: &Path) -> Result<(), OptifineError> {
    let java_path = get_java_binary(JavaVersion::Java21, JAVA, None).await?;
    let mut command = Command::new(&java_path);
    command
        .args([
            "-cp",
            &format!(
                "{}{CLASSPATH_SEPARATOR}.",
                new_installer_path.to_string_lossy()
            ),
            "OptifineInstaller",
        ])
        .current_dir(optifine_path);

    let output = command.output().path(java_path)?;
    if !output.status.success() {
        return Err(OptifineError::JavaFail(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ));
    }
    Ok(())
}

async fn compile_hook(
    new_installer_path: &Path,
    optifine_path: &Path,
    java_progress_sender: Option<&Sender<GenericProgress>>,
) -> Result<(), OptifineError> {
    let javac_path = get_java_binary(JavaVersion::Java21, "javac", java_progress_sender).await?;
    let mut command = Command::new(&javac_path);
    command
        .arg("-cp")
        .arg(new_installer_path.as_os_str())
        .args(["OptifineInstaller.java", "-d", "."])
        .current_dir(optifine_path);
    no_window!(command);

    let output = command.output().path(javac_path)?;
    if !output.status.success() {
        return Err(OptifineError::JavacFail(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ));
    }
    Ok(())
}

async fn create_details_json(instance_path: &Path) -> Result<(), OptifineError> {
    let details_path = instance_path.join("details.json");
    let details = tokio::fs::read_to_string(&details_path)
        .await
        .path(&details_path)?;
    let details: VersionDetails = serde_json::from_str(&details).json(details)?;

    let new_details_path = instance_path
        .join(".minecraft/versions")
        .join(details.get_id())
        .join(format!("{}.json", details.get_id()));

    tokio::fs::copy(&details_path, &new_details_path)
        .await
        .path(details_path)?;

    Ok(())
}

const OPTIFINE_ERR_PREFIX: &str = "while installing OptiFine:\n";

#[derive(Debug, Error)]
pub enum OptifineError {
    #[error("{OPTIFINE_ERR_PREFIX}{0}")]
    Io(#[from] IoError),
    #[error("{OPTIFINE_ERR_PREFIX}{0}")]
    JavaInstall(#[from] JavaInstallError),
    #[error("{OPTIFINE_ERR_PREFIX}The selected optifine installer file does not exist")]
    InstallerDoesNotExist,
    #[error("{OPTIFINE_ERR_PREFIX}could not compile installer\n\nSTDOUT = {0}\n\nSTDERR = {1}")]
    JavacFail(String, String),
    #[error("{OPTIFINE_ERR_PREFIX}could not run installer\n\nSTDOUT = {0}\n\nSTDERR = {1}")]
    JavaFail(String, String),
    #[error("{OPTIFINE_ERR_PREFIX}{0}")]
    Request(#[from] RequestError),
    #[error("{OPTIFINE_ERR_PREFIX}{0}")]
    Json(#[from] JsonError),
}

impl_3_errs_jri!(OptifineError, Json, Request, Io);
