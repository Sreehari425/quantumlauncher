//! A module to install Java from various third party sources
//! if Mojang doesn't provide Java for your specific platform.

use std::{
    env::consts::{ARCH, OS},
    io::Cursor,
    path::Path,
    sync::mpsc::Sender,
};

use ql_core::{file_utils, GenericProgress, JavaVersion};
use serde::Deserialize;

use crate::{extract_tar_gz, send_progress, JavaInstallError};

pub(crate) async fn install(
    version: JavaVersion,
    java_install_progress_sender: Option<&Sender<GenericProgress>>,
    install_dir: &Path,
) -> Result<(), JavaInstallError> {
    let url = get(version).await?;

    let Some(url) = url else {
        return Err(JavaInstallError::UnsupportedPlatform);
    };

    send_progress(
        java_install_progress_sender,
        GenericProgress {
            done: 0,
            total: 2,
            message: Some("Getting compressed archive".to_owned()),
            has_finished: false,
        },
    );
    let file_bytes = file_utils::download_file_to_bytes(&url, false).await?;
    send_progress(
        java_install_progress_sender,
        GenericProgress {
            done: 1,
            total: 2,
            message: Some("Extracting archive".to_owned()),
            has_finished: false,
        },
    );
    if url.ends_with("tar.gz") {
        extract_tar_gz(&file_bytes, install_dir).map_err(JavaInstallError::TarGzExtract)?;
    } else if url.ends_with("zip") {
        file_utils::extract_zip_archive(Cursor::new(file_bytes), &install_dir, true).await?;
    } else {
        return Err(JavaInstallError::UnknownExtension(url.to_owned()));
    }
    Ok(())
}

async fn get(mut version: JavaVersion) -> Result<Option<String>, JavaInstallError> {
    #[cfg(all(target_os = "freebsd", target_arch = "x86_64"))]
    if let JavaVersion::Java8 = version {
        return Ok(Some("https://github.com/Mrmayman/get-jdk/releases/download/java8-1/jdk-8u452-freebsd-x64.tar.gz".to_owned()));
    }
    if let JavaVersion::Java21 = version {
        if cfg!(all(target_os = "linux", target_arch = "arm")) {
            return Ok(Some("https://download.bell-sw.com/java/21.0.10+10/bellsoft-jdk21.0.10+10-linux-arm32-vfp-hflt.tar.gz".to_owned()));
        } else if cfg!(target_arch = "x86") {
            if cfg!(target_os = "windows") {
                return Ok(Some("https://download.bell-sw.com/java/21.0.10+10/bellsoft-jdk21.0.10+10-windows-i586.zip".to_owned()));
            } else if cfg!(target_os = "linux") {
                return Ok(Some("https://download.bell-sw.com/java/21.0.10+10/bellsoft-jdk21.0.10+10-linux-i586.tar.gz".to_owned()));
            }
        }
    }

    let mut res = get_inner(version).await?;
    while let (true, Some(next)) = (res.is_none(), version.next()) {
        // Try newer javas if older ones aren't there
        version = next;
        res = get_inner(version).await?;
    }
    return Ok(res);
}

#[derive(Deserialize)]
struct ZuluOut {
    latest: bool,
    download_url: String,
}

async fn get_inner(version: JavaVersion) -> Result<Option<String>, JavaInstallError> {
    let os = get_os();
    let arch = get_arch();

    let mut url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?java_version={version}&os={os}&arch={arch}&page_size=1000",
        version = version as usize
    );
    if let JavaVersion::Java21 = version {
        // For optifine
        url.push_str("&java_package_type=jdk");
    }
    let json: Vec<ZuluOut> = file_utils::download_file_to_json(&url, true).await?;
    let java = find_with_extension(&json, ".zip").or_else(|| find_with_extension(&json, ".tar.gz"));
    Ok(java.map(|n| n.download_url.clone()))
}

fn find_with_extension<'a>(json: &'a [ZuluOut], ext: &str) -> Option<&'a ZuluOut> {
    json.iter()
        .filter(|n| n.download_url.ends_with(ext))
        .find(|n| n.latest)
        .or_else(|| json.first())
}

fn get_os() -> &'static str {
    if cfg!(all(target_os = "linux", target_env = "gnu")) {
        "linux-glibc"
    } else if cfg!(all(target_os = "linux", target_env = "musl")) {
        "linux-musl"
    } else {
        OS
    }
}

fn get_arch() -> &'static str {
    if cfg!(target_arch = "arm") {
        "aarch32hf"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else if cfg!(all(target_arch = "sparc64", target_os = "solaris")) {
        "sparcv9-64"
    } else {
        ARCH
    }
}
