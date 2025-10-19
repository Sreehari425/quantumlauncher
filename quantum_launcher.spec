Name:           quantum-launcher
Version:        0.4.2
Release:        1%{?dist}
Summary:        Simple Minecraft Launcher written in Rust

License:        GPLv3
URL:            https://mrmayman.github.io/quantumlauncher
Source:         {{{ git_dir_pack }}}

BuildRequires:  rust cargo perl openssl-devel nasm

%global _description %{expand:
A simple Minecraft Launcher written in Rust.}

%description %{_description}

%prep
{{{ git_dir_setup_macro }}}
cargo fetch

%build
cargo build --profile release-ql

%install
install -Dm755 target/release-ql/quantum_launcher %{buildroot}%{_bindir}/quantum-launcher
install -Dm644 assets/freedesktop/quantum-launcher.desktop %{buildroot}%{_datadir}/applications/quantum-launcher.desktop
install -Dm644 assets/icon/256x256/ql_logo.png %{buildroot}%{_datadir}/pixmaps/io.github.Mrmayman.QuantumLauncher.png
install -Dm644 assets/icon/256x256/ql_logo.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.Mrmayman.QuantumLauncher.png
install -Dm644 assets/freedesktop/quantum-launcher.metainfo.xml %{buildroot}%{_datadir}/metainfo/quantum-launcher.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/quantum-launcher
%{_datadir}/applications/quantum-launcher.desktop
%{_datadir}/pixmaps/io.github.Mrmayman.QuantumLauncher.png
%{_datadir}/metainfo/quantum-launcher.metainfo.xml

%changelog
%autochangelog
