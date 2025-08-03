//! A module to install Java from various third party sources
//! (like Amazon Corretto) if Mojang doesn't provide Java for your specific platform.
//!
//! # Here is a table representing java platform support.
//!
//! - In this, any entry with numbers filled in represents "official"
//!   mojang support, ie. they provide the java install files
//!   for these platforms.
//!
//! - Any entry with __ represents a version not supported by
//!   mojang, but supported through *Amazon Corretto Java*
//!   which we provide an alternate installer for.
//!
//! - Any entry with -- represents a version not supported by
//!   mojang, but installed from
//!   <https://github.com/hmsjy2017/get-jdk>
//!
//! - Any entry with !! represents a version not supported at all.
//!
//! ```txt
//! linux x64 :  8 16 17 21
//! linux x32 :  8 !! !! !!  <- only java 8; MC 1.16.5 and below
//! linux a64 : __ __ __ __  <- corretto
//! linux a32 : -- !! !! !!  <- github
//!
//! macos x64 :  8 16 17 21
//! macos a64 : __ __ 17 21  <- corretto
//!
//! windw x64 :  8 16 17 21
//! windw x32 :  8 16 17 __  <- corretto
//! windw a64 : !! !! 17 21  <- only java 17+; mostly fine,
//!                             but some things like ModLoader might break
//! -------------------
//! windw means Windows
//! x64   means x86_64 (64 bit)
//! x32   means x86    (32 bit)
//! a64   means aarch64 or ARM (64 bit)
//! -------------------
//! ```
//!
//! So... yeah, enjoy this mess (WTF: )

use std::{io::Cursor, path::Path, sync::mpsc::Sender};

use ql_core::{file_utils, GenericProgress};

use crate::{extract_tar_gz, send_progress, JavaInstallError, JavaVersion};

pub async fn install(
    version: JavaVersion,
    java_install_progress_sender: Option<&Sender<GenericProgress>>,
    install_dir: &Path,
) -> Result<(), JavaInstallError> {
    #[allow(unused_mut)]
    let mut only_old_supported = false;
    let url;

    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        only_old_supported = true;
        url = "https://github.com/hmsjy2017/get-jdk/releases/download/v8u231/jdk-8u231-linux-arm32-vfp-hflt.tar.gz";
    }
    #[cfg(all(target_os = "solaris", target_arch = "x86_64"))]
    {
        only_old_supported = true;
        url = "https://github.com/hmsjy2017/get-jdk/releases/download/v8u231/jdk-8u231-solaris-x64.tar.gz";
    }
    #[cfg(all(target_os = "solaris", target_arch = "sparc64"))]
    {
        only_old_supported = true;
        url = "https://github.com/hmsjy2017/get-jdk/releases/download/v8u231/jdk-8u231-solaris-sparcv9.tar.gz";
    }

    if let JavaVersion::Java16 | JavaVersion::Java17 | JavaVersion::Java21 = version {
        if only_old_supported {
            return Err(JavaInstallError::UnsupportedOnlyJava8);
        }
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "arm"),
        all(
            target_os = "solaris",
            any(target_arch = "x86_64", target_arch = "sparc64")
        )
    )))]
    {
        url = version.get_corretto_url();
    }

    send_progress(
        java_install_progress_sender,
        GenericProgress {
            done: 0,
            total: 2,
            message: Some("Getting tar.gz archive".to_owned()),
            has_finished: false,
        },
    );
    let file_bytes = file_utils::download_file_to_bytes(url, false).await?;
    send_progress(
        java_install_progress_sender,
        GenericProgress {
            done: 1,
            total: 2,
            message: Some("Extracting tar.gz archive".to_owned()),
            has_finished: false,
        },
    );
    if url.ends_with("tar.gz") {
        extract_tar_gz(&file_bytes, install_dir).map_err(JavaInstallError::TarGzExtract)?;
    } else if url.ends_with("zip") {
        zip_extract::extract(Cursor::new(&file_bytes), install_dir, true)?;
    } else {
        return Err(JavaInstallError::UnknownExtension(url.to_owned()));
    }
    Ok(())
}

impl JavaVersion {
    #[must_use]
    pub(crate) fn get_corretto_url(self) -> &'static str {
        // https://aws.amazon.com/corretto/
        // for more info

        if cfg!(target_arch = "aarch64") && cfg!(target_os = "linux") {
            match self {
                JavaVersion::Java16 | JavaVersion::Java17 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-17-aarch64-linux-jdk.tar.gz"
                }
                JavaVersion::Java21 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-21-aarch64-linux-jdk.tar.gz"
                }
                JavaVersion::Java8 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-8-aarch64-linux-jdk.tar.gz"
                }
            }
        } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
            match self {
                JavaVersion::Java16 | JavaVersion::Java17 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-17-aarch64-macos-jdk.tar.gz"
                }
                JavaVersion::Java21 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-21-aarch64-macos-jdk.tar.gz"
                }
                JavaVersion::Java8 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-8-aarch64-macos-jdk.tar.gz"
                }
            }
        } else if cfg!(target_arch = "x86") && cfg!(target_os = "windows") {
            match self {
                JavaVersion::Java16 | JavaVersion::Java17 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-17-x86-windows-jdk.zip"
                }
                JavaVersion::Java21 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-21-x86-windows-jdk.zip"
                }
                JavaVersion::Java8 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-8-x86-windows-jdk.zip"
                }
            }
        } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "windows") {
            match self {
                JavaVersion::Java16 | JavaVersion::Java17 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-17-x64-windows-jdk.zip"
                }
                JavaVersion::Java21 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-21-x64-windows-jdk.zip"
                }
                JavaVersion::Java8 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-8-x64-windows-jdk.zip"
                }
            }
        } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") {
            match self {
                JavaVersion::Java16 | JavaVersion::Java17 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-17-x64-linux-jdk.zip"
                }
                JavaVersion::Java21 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-21-x64-linux-jdk.zip"
                }
                JavaVersion::Java8 => {
                    "https://corretto.aws/downloads/latest/amazon-corretto-8-x64-linux-jdk.tar.gz"
                }
            }
        } else {
            panic!("Unsupported OS")
        }
    }
}
