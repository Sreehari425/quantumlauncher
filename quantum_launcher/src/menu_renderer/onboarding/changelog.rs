use iced::widget;

use crate::menu_renderer::{onboarding::IMG_MANAGE_MODS, Element};

#[allow(unused)]
pub fn changelog<'a>() -> Element<'a> {
    const FS: u16 = 14;

    widget::column![
        widget::text("Welcome to QuantumLauncher v0.4.3!").size(40),

        widget::text("You can now install OptiFine and Forge together!"),

        widget::column![
            widget::text("Added alternate fabric implementations for versions without official Fabric support:"),
            widget::text("- Legacy Fabric (1.3-1.13)").size(14),
            widget::text("- OrnitheMC (b1.7-1.13)").size(14),
            widget::text("- Babric and Cursed Legacy (b1.7.3)").size(14),
        ].spacing(5),

        widget::horizontal_rule(1),
        widget::text("UX").size(32),

        widget::column![
        "- Overhauled the mod list, now with ICONS, bulk-selection, and better aesthetics.",
        "- Export mods as a text list for easy manual sharing, with optional links and instance details",
        "- Tweaked and rearranged many menus/messages",
        ].spacing(5),

        widget::image(IMG_MANAGE_MODS.clone()).height(400),

        widget::horizontal_rule(1),
        widget::text("Keyboard Navigation").size(32),

        widget::column![
            "- \"Ctrl/Cmd/Alt 1/2/3\" to switch tabs in main screen",
            "- \"Ctrl N\" to create new instance",
            "- \"Ctrl ,\" to open settings",
        ].spacing(5),

        widget::horizontal_rule(1),
            widget::text("Fixes").size(32),
        if true { widget::column![
            widget::container(
                widget::text("NOTE: On linux, the files location has been moved from\n~/.config to ~/.local/share (with auto-migration via symlinks)").size(13),
            ).padding(5),
            widget::Space::with_height(5),
            widget::text("- Colored terminal output on Windows.").size(14),
            widget::text("- CurseForge mods without a loader can now be installed.").size(14),
            widget::text("- Instances from newer launcher versions can be opened in v0.4.1.").size(14),
            widget::text("- Backspace no longer kills running instances without Ctrl.").size(14),
            widget::text("- Added warning if xrandr isn't installed").size(14),
            widget::Space::with_height(5),
            widget::text("- Fixed the game log being a single-line mess ").size(14),
            widget::text("- Fixed \"java binary not found\" macOS error.").size(14),
            widget::text("- Fixed crash with \"Better Discord Rich Presence\" mod.").size(14),
            widget::text("- Fixed launcher panic when launching the game.").size(14),
            widget::text("- Fixed NeoForge 1.21.1 and Forge 1.21.5 crash (reinstall loader to apply)").size(14),
            widget::text("- Fixed forge installer error: \"Processor failed, invalid outputs\"").size(14),
            widget::text("- Fixed \"SSLHandshakeException\" crash on Windows").size(14),
        ].spacing(5) } else { widget::Column::new() },

        widget::Space::with_height(10),
        widget::container(widget::text("By the way, I've been busy with my life a lot lately.\nSorry for the lack of features.").size(12)).padding(10),
        widget::Space::with_height(10),
        widget::text("Ready to experience your new launcher now? Hit continue!").size(20),
    ]
    .padding(10)
    .spacing(10)
    .into()
}
