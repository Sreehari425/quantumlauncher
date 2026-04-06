use iced::widget::{self, column};

use crate::{
    config::LauncherConfig,
    menu_renderer::{Column, checkered_list, tsubtitle},
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
                widget::text("Basic Presence String"),
                widget::text("Changes will take effect with a toggle of the feature / launcher restart.").size(12).style(tsubtitle),
                widget::text_input("Enter presence text...", &self.default_presence_string)
                    .on_input(|v| Message::LauncherSettings(
                        LauncherSettingsMessage::DefaultPresenceStringChanged(v)
                    )),
            ].spacing(5)
        ])
    }
}
