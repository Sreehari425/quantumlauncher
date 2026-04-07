use iced::{
    Length,
    widget::{self, column, row},
};

use crate::{
    config::LauncherConfig,
    icons,
    menu_renderer::{Column, button_with_icon, checkered_list, tsubtitle},
    state::{LauncherSettingsMessage, MenuLauncherSettings, Message},
};

impl MenuLauncherSettings {
    pub(super) fn view_presence_tab<'a>(&'a self, config: &'a LauncherConfig) -> Column<'a> {
        checkered_list([
            column![
                row![
                    widget::text("Discord Rich Presence").size(20).width(Length::Fill),
                    button_with_icon(icons::refresh_s(14), "Reset to Defaults", 14)
                        .on_press_with(|| Message::LauncherSettings(LauncherSettingsMessage::ResetPresence)),
                ]
            ],

            column![
                widget::checkbox("Enable Broadcast", config.rich_presence.unwrap_or(true))
                    .on_toggle(|n| Message::LauncherSettings(
                        LauncherSettingsMessage::ToggleDiscordRichPresence(n)
                    )),
                widget::text("Sometimes toggling this option might take some time to apply the activity updates on Discord.").size(12).style(tsubtitle),
                widget::Space::with_height(5),
                    if self.is_presence_running {
                        row!(
                            icons::version_tick_s(13),
                            widget::Space::with_width(5),
                            widget::text("Synced!").size(13).style(tsubtitle)
                        )
                    } else if config.rich_presence.unwrap_or(true) {
                        row!(
                            icons::clock_s(13),
                            widget::Space::with_width(5),
                            widget::text("Awaiting sync...").size(13).style(tsubtitle),
                        )
                    } else {
                        row!(
                            icons::cross_s(13),
                            widget::Space::with_width(5),
                            widget::text("Not enabled.").size(13).style(tsubtitle)
                        )
                    }

            ]
            .spacing(5),

            column![
                widget::text("Custom Presence:"),
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
                if config.rich_presence.unwrap_or(true) {
                    widget::column![
                        widget::Space::with_height(5),
                        button_with_icon(icons::discord_s(16), "Set Now", 12)
                            .padding([5, 10])
                            .on_press(Message::LauncherSettings(LauncherSettingsMessage::SetPresenceNow))
                    ]
                } else {
                    widget::column![]
                },
            ].spacing(5),

            column![
                widget::text("Toggles:"),
                widget::Space::with_height(5),
                widget::checkbox("Change presence during play/quit events", config.rich_presence_events.unwrap_or(true)).on_toggle(|n| Message::LauncherSettings(LauncherSettingsMessage::TogglePresenceEvents(n))),
                widget::text("Disabling this will ensure that only the custom rich presence set above stays alive when you run the launcher and/or play Minecraft.").size(12).style(tsubtitle),

            ].spacing(5),

            if config.rich_presence_events.unwrap_or(true) {
                widget::column![
                    widget::text("Event Presences:"),
                    widget::text("NOTE: You can use substitutes like ${instance} and ${version} for instance and version names respectively.").size(12).style(tsubtitle),
                    widget::Space::with_height(6),
                    widget::text("Game Launch").size(14),
                    widget::Space::with_height(3),
                    widget::text_input("*Enter top text...", &self.gameopen_presence_details)
                        .on_input(|v| Message::LauncherSettings(
                            LauncherSettingsMessage::GameOpenPresenceDetailsChanged(v)
                        )),
                    widget::text_input("Enter bottom text...", &self.gameopen_presence_state)
                        .on_input(|v| Message::LauncherSettings(
                            LauncherSettingsMessage::GameOpenPresenceStateChanged(v)
                        )),
                    widget::Space::with_height(3),
                    widget::text("Game Exit").size(14),
                    widget::Space::with_height(3),
                    widget::text_input("*Enter top text...", &self.gameexit_presence_details)
                        .on_input(|v| Message::LauncherSettings(
                            LauncherSettingsMessage::GameExitPresenceDetailsChanged(v)
                        )),
                    widget::text_input("Enter bottom text...", &self.gameexit_presence_state)
                        .on_input(|v| Message::LauncherSettings(
                            LauncherSettingsMessage::GameExitPresenceStateChanged(v)
                        )),

                ].spacing(5)
            } else {
                widget::column![]
            },
        ])
    }
}
