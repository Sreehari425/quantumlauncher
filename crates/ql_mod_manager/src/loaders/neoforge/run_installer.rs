use std::{path::Path, sync::mpsc::Sender};

use ql_core::{no_window, pt, GenericProgress, IntoIoError, CLASSPATH_SEPARATOR};
use ql_java_handler::{get_java_binary, JavaVersion};
use tokio::{fs, process::Command};

use crate::loaders::{
    forge::{ForgeInstallError, ForgeInstallProgress},
    neoforge::{send_progress, INSTALLER_NAME},
    FORGE_INSTALLER_CLIENT, FORGE_INSTALLER_SERVER,
};

pub async fn compile_and_run_installer(
    neoforge_dir: &Path,
    j_progress: Option<&Sender<GenericProgress>>,
    f_progress: Option<&Sender<ForgeInstallProgress>>,
    is_server: bool,
) -> Result<(), ForgeInstallError> {
    pt!("Running Installer");
    send_progress(f_progress, ForgeInstallProgress::P4RunningInstaller);

    let installer = if is_server {
        FORGE_INSTALLER_SERVER
    } else {
        FORGE_INSTALLER_CLIENT
    };
    let installer_class = neoforge_dir.join("ForgeInstaller.class");
    fs::write(&installer_class, installer)
        .await
        .path(installer_class)?;

    let java_path = get_java_binary(JavaVersion::Java21, "java", j_progress).await?;
    let mut command = Command::new(&java_path);
    no_window!(command);
    command
        .args([
            "-cp",
            &format!(
                "forge/{INSTALLER_NAME}{CLASSPATH_SEPARATOR}{INSTALLER_NAME}{CLASSPATH_SEPARATOR}forge/{CLASSPATH_SEPARATOR}."
            ),
            "ForgeInstaller",
        ])
        .current_dir(if is_server {
            neoforge_dir
                .parent()
                .map_or(neoforge_dir.join(".."), |n| n.to_owned())
        } else {
            neoforge_dir.to_owned()
        });

    let output = command.output().await.path(java_path)?;
    if !output.status.success() {
        return Err(ForgeInstallError::InstallerError(
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
        ));
    }
    Ok(())
}
