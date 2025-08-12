# Unreleased

# TLDR;

- Revamped and improved many menus, plus a new Teal theme!
- Added [`ely.by`](https://ely.by) and [`littleskin`](https://littleskin.cn) account integration

- Switched to [BetterJSONs](https://github.com/MCPHackers/BetterJSONs/)
  and [LaunchWrapper](https://github.com/MCPHackers/LaunchWrapper): Fixing many issues (marked with `(b)`)
- Many, **many** bug-fixes and performance improvements! (for real)

---

# [`ely.by`](https://ely.by)/[`littleskin`](https://littleskin.cn) integration

- You can now log in with [`ely.by`](https://ely.by) and [`littleskin`](https://littleskin.cn) accounts!
- Minecraft 1.21.5 and below support skins from both services `(b)`

> **Note**:
> You'll need to create a new instance for skins to work without mods.
>
> For existing instances, and for 1.21.6+, use the CustomSkinLoader mod

# UI

## Revamped:

- Launcher settings (redesign + licenses page + UI antialiasing option)
- Confirmation screen
- All launcher icons (thanks, [Aurlt](https://github.com/Aurlt) !)

## Improved:

- Instance edit menu
- OptiFine install menu (now with Drag & Drop, delete installer option)

# CLI

The following commands have been added:

- `quantum_launcher create <NAME> <VERSION>` (add `-s` to skip downloading assets (music/sound))
- `quantum_launcher launch <INSTANCE> <USERNAME>` (add `-s` for account authentication)
- `quantum_launcher delete <INSTANCE>` (add `-f` to skip confirmation)

# System & Platform

- Overhauled portable/custom directory system (see `docs/PORTABLE.md` for more info)
- New (incomplete) support for: FreeBSD, Windows 7, Linux ARM 32-bit

# Fixes

- Fixed many crashes on Linux ARM and macOS `(b)`
- Fixed game crashes in portable mode
- Fixed many formatting issues in game logs
- Fixed welcome screen not working
- Fixed arrow keys to switch instances, not updating the Edit menu

- Improved readability of a few errors
- Improved support for weird character encodings in file paths
- Missing libraries are now auto-downloaded
- Last account selected is now remembered

## Modding

- Fixed Fabric API being missing for some curseforge mods
- Fixed getting stuck in an infinite loop when downloading some curseforge mods

## Versions

- Fixed Minecraft Indev and early Infdev being unplayable `(b)`
- Fixed broken colors in old versions on M-series Macs `(b)`
- Old Minecraft versions are now in the correct order in the download list `(b)`
- Snapshots of 1.0 to 1.5.2 are no longer missing for download `(b)`

## Performance

- Fixed lag spikes on some systems when selecting instances
- Many autosaving features has been slowed down, and disk accesses reduced
- Optimized the log renderer (slightly worse scrolling as a tradeoff)
- The "Create Instance" version list loads **way** faster now `(b)`
