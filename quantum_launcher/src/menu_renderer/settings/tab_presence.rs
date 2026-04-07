use iced::widget::{self, column};

use crate::{
    config::LauncherConfig,
    icons,
    menu_renderer::{Column, button_with_icon, checkered_list, tsubtitle},
    state::{LauncherSettingsMessage, MenuLauncherSettings, Message},
};

impl MenuLauncherSettings {
    pub(super) fn view_presence_tab<'a>(&'a self, config: &'a LauncherConfig) -> Column<'a> {
        checkered_list([
            column![widget::text("Discord Rich Presence").size(20)],

            column![
                widget::checkbox("Enable Broadcast", config.rich_presence.unwrap_or(true))
                    .on_toggle(|n| Message::LauncherSettings(
                        LauncherSettingsMessage::ToggleDiscordRichPresence(n)
                    )),
                widget::text("Sometimes toggling this option might take some time to apply the activity updates on Discord.").size(12).style(tsubtitle),

            ]
            .spacing(5),

            column![
                widget::text("Custom Presence"),
                widget::text("Changes will take effect on launcher restart or with the press of the button below.").size(12).style(tsubtitle),
                widget::Space::with_height(5),
                widget::text_input("*Enter top text...", &self.default_presence_details)
                    .on_input(|v| Message::LauncherSettings(
                        LauncherSettingsMessage::DefaultPresenceDetailsChanged(v)
                    )),
                widget::text_input("Enter bottom text...", &self.default_presence_state)
                    .on_input(|v| Message::LauncherSettings(
                        LauncherSettingsMessage::DefaultPresenceStateChanged(v)
                    )),
                widget::Space::with_height(5),
                button_with_icon(icons::discord_s(16), "Set Now", 12)
                    .padding([5, 10])
                    .on_press(Message::LauncherSettings(LauncherSettingsMessage::SetPresenceNow)),
            ].spacing(5),

            column![
                widget::text("Toggles"),
                widget::Space::with_height(5),
                widget::checkbox("Change presence during play/quit events", config.rich_presence_events.unwrap_or(true)).on_toggle(|n| Message::LauncherSettings(LauncherSettingsMessage::TogglePresenceEvents(n))),
                widget::text("Disabling this will ensure that only the custom rich presence set above stays alive when you run the launcher and/or play Minecraft.").size(12).style(tsubtitle),
                widget::Space::with_height(5),
                widget::checkbox("Show current instance name", config.rich_presence_show_instance_name.unwrap_or(true)).on_toggle(|n| Message::LauncherSettings(LauncherSettingsMessage::TogglePresenceShowInstanceName(n))),
                widget::Space::with_height(5),
                widget::checkbox("Show Minecraft version", config.rich_presence_show_minecraft_version.unwrap_or(true)).on_toggle(|n| Message::LauncherSettings(LauncherSettingsMessage::TogglePresenceShowMinecraftVersion(n))),
            ].spacing(5)
        ])
    }
}
