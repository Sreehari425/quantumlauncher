Command: `cargo run -p tests`

This launches an instance, and closes the window if successful.
It scans for a window for `timeout` seconds.

Cli Flags (optional):

- `--help`: See help message
- `--existing`: Whether to reuse existing Minecraft files instead of redownloading them
- `--timeout`: How long to wait (in seconds) for a window before giving up (default: 60).
  You may want to increase this on slower systems
- `--verbose`: See all the logs to diagnose issues
- `--skip-lwjgl3`: Only tests legacy LWJGL2-based versions (1.12.2 and below).
  Useful for less supported platforms like FreeBSD
- `--skip-loaders` (TODO): Only test vanilla Minecraft, skipping mod loaders

# Supports

- Windows
- X11 (or XWayland) like **Linux, FreeBSD**, etc
- macOS (experimental)

# TODO

- Add proper macOS support
- Test different fabric backends
- Add `--include`/`--exclude` flags to select specific versions to test
