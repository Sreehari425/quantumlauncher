use cfg_if::cfg_if;

cfg_if!(
    if #[cfg(any(feature = "simulate_linux_arm64", feature = "simulate_linux_arm32"))] {
        pub const OS_NAME: &str = "linux";
        pub const OS_NAMES: &[&str] = &["linux"];
    } else if #[cfg(feature = "simulate_macos_arm64")] {
        pub const OS_NAME: &str = "osx";
        pub const OS_NAMES: &[&str] = &["macos", "osx"];
    } else if #[cfg(target_os = "linux")] {
        pub const OS_NAME: &str = "linux";
        pub const OS_NAMES: &[&str] = &["linux"];
    } else if #[cfg(target_os = "macos")] {
        pub const OS_NAME: &str = "osx";
        pub const OS_NAMES: &[&str] = &["macos", "osx"];
    } else if #[cfg(target_os = "windows")] {
        pub const OS_NAME: &str = "windows";
        pub const OS_NAMES: &[&str] = &["windows"];
    } else if #[cfg(target_os = "freebsd")] {
        pub const OS_NAME: &str = "freebsd";
        pub const OS_NAMES: &[&str] = &["freebsd"];
    }
);

pub const DEFAULT_RAM_MB_FOR_INSTANCE: usize = 2048;

cfg_if!(
    if #[cfg(any(
        feature = "simulate_linux_arm64",
        feature = "simulate_macos_arm64"
    ))] {
        pub const ARCH: &str = "arm64";
    } else if #[cfg(feature = "simulate_linux_arm32")] {
        pub const ARCH: &str = "arm32";
    } else if #[cfg(target_arch = "aarch64")] {
        pub const ARCH: &str = "arm64";
    } else if #[cfg(target_arch = "arm")] {
        pub const ARCH: &str = "arm32";
    } else if #[cfg(target_arch = "x86")] {
        pub const ARCH: &str = "x86";
    }
);
