use iced::widget::{self, column, text};

use crate::menu_renderer::Element;

pub fn changelog() -> Element<'static> {
    column![
        text("Welcome to QuantumLauncher v0.5.1!").size(40),

        widget::container(column![
            "TLDR;",
            text("- Instance folders for better organization").size(14),
            text("- One-click shortcuts: launch without opening the launcher!").size(14),
            text("- Numerous UX improvements and bug fixes").size(14),
        ].spacing(5)).padding(10),

        text("Instance Folders").size(32),
        column![
            "Organize and sort large collections easily, with instance folders!",
            text("Complete with drag-and-drop, renaming and nesting").size(14),
            widget::container(
                column![
                    "Note:",
                    text("- Folders are purely organizational and do not affect file paths on disk").size(12),
                    text("- Custom instance icons had to be delayed to a future release due to time constraints").size(12),
                ].padding(10),
            ),
        ].spacing(5),

        text("Shortcuts").size(32),
        column![
            "Launch instances with a single click, without opening the launcher!",
            "Create one-click shortcuts for:",
            column![
                text("- Desktop").size(14),
                text("- Start Menu / Applications Menu (Windows & Linux)").size(14),
                text("- Applications folder (macOS)").size(14),
                text("- Custom locations").size(14),
            ],
            "Custom icons had to be delayed to a future release due to time constraints",
        ].spacing(10),

        widget::horizontal_rule(1),
        text("UX").size(32),

        column![
            "- Improved Welcome screen with keyboard navigation, cleaner layout, and clearer guidance for new users",
            "- Increased maximum memory allocation to 32 GB in the Edit tab",
            text("  - Added precise manual input alongside the slider").size(14),
        ].spacing(5),

        text("Mod Menu").size(20),
        column![
            "- Added quick uninstall button in Mod Store",
            "- More visible enable/disable toggle in mod list",
            "- Disabled mods now remain disabled after updates",
        ].spacing(5),

        text("Logging:").size(20),
        column![
            "- Overhauled log viewer:",
            text(" - Text selection support").size(14),
            text(" - Smoother scrolling").size(14),
            text(" - Fewer bugs").size(14),
            "- Fixed missing crash reports in logs",
            "- Added warning when running inside a macOS VM",
        ].spacing(5),

        widget::horizontal_rule(1),
        text("Technical").size(32),
        column![
            "- Mod update checks are now manual",
            text("  - Use \"… → Check for Updates\"").size(14),
            text("  - Reduces network usage and prevents frequent 504 errors").size(14),
            widget::Space::with_height(5),
            "- Usernames are now redacted in log paths",
            text("  - Example: `C:\\Users\\YOUR_NAME` → `C:\\Users\\[REDACTED]`").size(14),
            text("  - Disable temporarily with `--no-redact-info`").size(14),
            widget::Space::with_height(10),
            text("CLI").size(24),
            text("A few command-line flags were added to `quantum_launcher launch <INSTANCE> <USERNAME>`").size(14),
            text("--show-progress").size(20),
            text("  - Displays desktop notifications for login progress and errors").size(14),
            text("  - Especially useful for shortcuts and scripts").size(14),
            text("--account-type").size(20),
            text("  - Manually specify account type for login (eg: `microsoft`, `elyby`, `littleskin`)").size(14),
            text("  - Useful if you have multiple accounts with the same name").size(14),
            widget::Space::with_height(10),
            text("Java").size(24),
            "- Choose launcher-managed Java versions or custom paths",
            "- Improved Java installer with broader platform support",
            text("  - Minecraft 1.20.5–1.21.11 now runs on many 32-bit systems").size(14),
            "- Platforms without Mojang Java now use Azul Zulu instead of Amazon Corretto",
            "- Java override in Edit tab now supports selecting folders",
        ].spacing(5),

        widget::horizontal_rule(1),
        text("Fixes").size(20),
        column![
            text("- Fixed critical issue preventing game downloads (caused by a BetterJSONs API breaking change)").size(12),
            text("- Fixed Modrinth \"error code 504\" issues from automatic update checks").size(12),
            text("- Fixed context menus not closing after a click").size(12),
            text("- Fixed several CurseForge concurrent download issues").size(12),
            text("- Fixed QMP presets added via \"Add File\" not installing all mods").size(12),
            text("- Fixed account login persistence for new users").size(12),
            text("- Fixed post-1.21.11 versions failing to launch on Linux ARM").size(12),
            text("- Fixed unnecessary Java re-downloads on some ARM systems").size(12),
            text("- Fixed duplicate-named mods causing glitches in the store (e.g., multiple \"Clumps\" mods)").size(12),
        ].spacing(5),

        widget::Space::with_height(10),
        widget::container(text(r"Final Note: I had plans for a lot more, but had to force
this out in a matter of days due to the critical BetterJSONs bug.").size(12)).padding(10),
        widget::Space::with_height(10),
        text("Ready to experience your new launcher now? Hit continue!").size(20),
    ]
    .padding(10)
    .spacing(10)
    .into()
}
