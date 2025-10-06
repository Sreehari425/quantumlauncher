Plans for the future of QuantumLauncher.

# Core

- [x] Instance creation, editing, launching
- [x] Optional authentication with Microsoft, `ely.by`, `littleskin`
- [x] Integration with Omniarchive, old version support
- [ ] Full controller, keyboard-navigation support in UI

# Instances

- [ ] Import MultiMC/PrismLauncher instances
- [ ] Migrate from other launchers
- [ ] Package QuantumLauncher instances (WIP)
- [ ] Upgrading instances to a newer Minecraft version

#

---

# Mods

## Loaders

- [x] Fabric
- [x] Forge
- [x] Optifine
- [x] Quilt
- [x] Neoforge
- [ ] OptiForge (WIP)
- [ ] OptiFabric
- [ ] Other Fabric implementations (WIP)
- [x] Jar Mods

## Sources

- [x] Modrinth mods
- [x] Curseforge mods
- [x] Modrinth modpacks
- [x] Curseforge modpacks

## Features

- [x] Mod store
- [x] Mod presets (packaging mods)
- [x] Mod updater
- [ ] Modpack UI/UX improvements
- [ ] Filters in Mod store

---

# Servers

The server manager is highly incomplete and under
active development, so it's temporarily disabled.

This will allow you to setup and host Minecraft servers
across the internet from a single click. Think Aternos
but local and ad-free.

- [x] Create/delete/run Minecraft servers
- [x] Editing basic server settings (RAM, Java, Args)
- [ ] Editing `server.properties`
- [ ] Editing NBT config files
- [ ] Plugin store
- [ ] [playit.gg](https://playit.gg) integration
- [ ] Version-control based world rollback system

## Loaders

- [x] Paper
- [ ] Spigot
- [ ] Bukkit
- [ ] Bungeecoord
- [ ] [Combining mod-loaders and plugin-loaders](https://github.com/LeStegii/server-software/blob/master/java/MODS+PLUGINS.md)

---

# Platforms

> Everything **not highlighted in bold**
> isn't 100% guaranteed to work, due to lack of development resources

- [x] **Windows x86_64**
- [x] **Linux x86_64**
- [x] macOS x86_64

- [x] Windows Aarch64
- [x] Linux Aarch64
- [x] Linux ARM32
- [x] macOS Aarch64

- [x] Windows i686
- [ ] Linux i686

- [x] FreeBSD (WIP, Minecraft 1.12.2 and below)
- [ ] Haiku
- [ ] Solaris
- [ ] Android (in the distant future)

# Command-Line interface

- [x] `list-instances`, `-l`
- [x] `list-available-versions`, `-a`
- [x] `create NAME VERSION`
- [x] `launch INSTANCE USERNAME`
- [ ] Install loaders from CLI
- [ ] Mod installation features from CLI
- [ ] Preset, modpack features from CLI

# Misc
- [ ] Plugin system in lua ([abandoned implementation here](https://github.com/Mrmayman/quantumlauncher/blob/16e02b1e36a736fadb3214b84de908eb21635a55/plugins/README.md), scrapped due to complexity)
