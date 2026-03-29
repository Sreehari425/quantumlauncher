use std::sync::LazyLock;

use iced::{Alignment, Length, widget};
use ql_core::{LAUNCHER_DIR, WEBSITE};

use super::{
    DISCORD, Element, GITHUB, back_button, button_with_icon, get_mode_selector, sidebar_button,
    underline,
};
use crate::menu_renderer::edit_instance::{args_split_by_space, get_args_list, resolution_dialog};
use crate::menu_renderer::{back_to_launch_screen, checkered_list, sidebar, tsubtitle};
use crate::{
    config::LauncherConfig,
    icons,
    state::{LauncherSettingsMessage, LauncherSettingsTab, MenuLauncherSettings, Message},
    stylesheet::{
        color::Color,
        styles::{LauncherTheme, LauncherThemeColor},
        widgets::StyleButton,
    },
};

pub static IMG_ICED: LazyLock<widget::image::Handle> = LazyLock::new(|| {
    widget::image::Handle::from_bytes(include_bytes!("../../../assets/iced.png").as_slice())
});

const SETTINGS_SPACING: f32 = 10.0;
const SETTING_WIDTH: u16 = 180;

pub const PREFIX_EXPLANATION: &str =
    "Commands to add before the game launch command\nEg: prime-run/gamemoderun/mangohud";

impl MenuLauncherSettings {
    pub fn view<'a>(&'a self, config: &'a LauncherConfig) -> Element<'a> {
        widget::row![
            sidebar(
                "MenuLauncherSettings:sidebar",
                Some(
                    widget::column![
                        back_button().on_press(back_to_launch_screen(None, None)),
                        Self::get_heading()
                    ]
                    .spacing(10)
                    .into()
                ),
                LauncherSettingsTab::ALL.iter().map(|tab| {
                    let text = widget::text(tab.to_string());
                    sidebar_button(
                        tab,
                        &self.selected_tab,
                        text,
                        LauncherSettingsMessage::ChangeTab(*tab).into(),
                    )
                })
            )
            .style(|_: &LauncherTheme| widget::container::Style {
                text_color: None,
                background: None,
                border: iced::Border::default(),
                shadow: iced::Shadow::default()
            }),
            widget::scrollable(self.selected_tab.view(config, self))
                .width(Length::Fill)
                .spacing(0)
                .style(LauncherTheme::style_scrollable_flat_dark)
        ]
        .into()
    }

    fn get_heading() -> widget::Row<'static, Message, LauncherTheme> {
        widget::row![icons::gear_s(20), widget::text("Settings").size(20)]
            .padding(iced::Padding {
                top: 5.0,
                right: 0.0,
                bottom: 2.0,
                left: 10.0,
            })
            .spacing(10)
    }

    fn view_ui_tab<'a>(&'a self, config: &'a LauncherConfig) -> Element<'a> {
        let ui_scale_apply = widget::row![
            widget::horizontal_space(),
            widget::button(widget::text("Apply").size(12))
                .padding([1.8, 5.0])
                .on_press(Message::LauncherSettings(
                    LauncherSettingsMessage::UiScaleApply,
                ))
        ];

        let idle_fps = config.c_idle_fps();

        checkered_list::<Element>([
            widget::column![widget::text("User Interface").size(20)].into(),

            widget::column![
                widget::row!["Mode: ", get_mode_selector(config)]
                    .spacing(5)
                    .align_y(Alignment::Center),
                widget::Space::with_height(5),
                widget::row!["Theme:", get_theme_selector().wrap()].spacing(5),
            ]
            .spacing(5)
            .into(),
            widget::row![
                widget::row![widget::text!("UI Scale ({:.2}x)  ", self.temp_scale).size(15)]
                    .push_maybe(
                        ((self.temp_scale - config.ui_scale.unwrap_or(1.0)).abs() > 0.01)
                            .then_some(ui_scale_apply)
                    )
                    .align_y(Alignment::Center).width(SETTING_WIDTH),
                widget::slider(0.5..=2.0, self.temp_scale, |n| Message::LauncherSettings(
                    LauncherSettingsMessage::UiScale(n)
                ))
                .step(0.1),
            ]
            .align_y(Alignment::Center)
            .spacing(5)
            .into(),

            get_ui_opacity(config).into(),

            widget::column![
                // TODO: This requires launcher restart
                // widget::checkbox("Custom Window Decorations", !config.c_window_decorations()).on_toggle(|n| {
                //     LauncherSettingsMessage::ToggleWindowDecorations(n).into()
                // }),
                // widget::text("Use custom window borders and close/minimize/maximize buttons").size(12),
                // widget::Space::with_height(5),

                widget::checkbox("Antialiasing (UI) - Requires Restart", config.ui_antialiasing.unwrap_or(true))
                    .on_toggle(|n| Message::LauncherSettings(
                        LauncherSettingsMessage::ToggleAntialiasing(n)
                    )),
                widget::text("Makes text/menus crisper. Also nudges the launcher into using your dedicated GPU for the User Interface").size(12).style(tsubtitle),
                widget::Space::with_height(5),

                widget::checkbox("Remember window size", config.window.as_ref().is_none_or(|n| n.save_window_size))
                    .on_toggle(|n| LauncherSettingsMessage::ToggleWindowSize(n).into()),
                widget::Space::with_height(5),
                widget::checkbox("Remember last selected instance", config.persistent.clone().unwrap_or_default().selected_remembered)
                    .on_toggle(|n| LauncherSettingsMessage::ToggleInstanceRemembering(n).into()),
            ]
            .spacing(5)
            .into(),

            widget::column![
                widget::row![
                    widget::text!("UI Idle FPS ({idle_fps})")
                        .size(15)
                        .width(SETTING_WIDTH),
                    widget::slider(2.0..=20.0, idle_fps as f64, |n| Message::LauncherSettings(
                        LauncherSettingsMessage::UiIdleFps(n)
                    ))
                    .step(1.0).shift_step(1.0),
                ]
                .align_y(Alignment::Center)
                .spacing(5),
                widget::text(r#"(Default: 6) Reduces resource usage when launcher is idle.
Only increase if progress bars stutter or "not responding" dialogs show"#).size(12).style(tsubtitle),
            ].spacing(5).into()
        ])
        .into()
    }

    fn view_location_tab<'a>(&'a self, _config: &'a LauncherConfig) -> Element<'a> {
        let t = |s| widget::text(s).size(12).style(tsubtitle);

        let current_dir_text = widget::text(format!("{}", LAUNCHER_DIR.display())).size(14);

        checkered_list::<Element>([
            widget::column![widget::text("Location").size(20)].into(),
            widget::column![
                widget::text("Current Data Directory").size(16),
                t("This is the active folder where all your Minecraft instances, mods, and launcher settings are saved."),
                widget::Space::with_height(5),
                widget::row![
                    current_dir_text.width(Length::Fill),
                    button_with_icon(icons::folder(), "Open Folder", 13)
                        .on_press(Message::CoreOpenPath(LAUNCHER_DIR.clone())),
                ].align_y(Alignment::Center).spacing(10),
            ]
            .spacing(5)
            .into(),
            view_portable_mode_section(self).into(),
            view_system_redirect_section(self).into(),
        ])
        .into()
    }
}

fn get_ui_opacity(config: &LauncherConfig) -> widget::Column<'static, Message, LauncherTheme> {
    let ui_opacity = config.c_ui_opacity();
    let t = |t| widget::text(t).size(12).style(tsubtitle);

    widget::column![
        widget::row![
            widget::text!("Window Opacity ({ui_opacity:.2}x)")
                .width(SETTING_WIDTH)
                .size(15),
            widget::slider(0.5..=1.0, ui_opacity, |n| Message::LauncherSettings(
                LauncherSettingsMessage::UiOpacity(n)
            ))
            .step(0.1)
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        t("Window background transparency\n(May not work on all systems/GPUs)"),
        t("0.5 (translucent) ..  1.0 (opaque)"),
    ]
    .spacing(5)
}

pub fn get_theme_selector() -> widget::Row<'static, Message, LauncherTheme> {
    widget::row(LauncherThemeColor::ALL.iter().map(|color| {
        widget::button(widget::text(color.to_string()).size(13))
            .padding([2, 4])
            .style(|theme: &LauncherTheme, s| {
                LauncherTheme {
                    color: *color,
                    alpha: 1.0,
                    ..*theme
                }
                .style_button(s, StyleButton::Round)
            })
            .on_press(Message::LauncherSettings(
                LauncherSettingsMessage::ColorSchemePicked(*color),
            ))
            .into()
    }))
    .spacing(5)
}

impl LauncherSettingsTab {
    pub fn view<'a>(
        &'a self,
        config: &'a LauncherConfig,
        menu: &'a MenuLauncherSettings,
    ) -> Element<'a> {
        match self {
            LauncherSettingsTab::UserInterface => menu.view_ui_tab(config),
            LauncherSettingsTab::Location => menu.view_location_tab(config),
            LauncherSettingsTab::Internal => widget::column![
                widget::text("Game").size(20),
                resolution_dialog(
                    config.global_settings.as_ref(),
                    |n| Message::LauncherSettings(
                        LauncherSettingsMessage::DefaultMinecraftWidthChanged(n)
                    ),
                    |n| Message::LauncherSettings(
                        LauncherSettingsMessage::DefaultMinecraftHeightChanged(n)
                    ),
                ),
                widget::horizontal_rule(1),
                "Global Java Arguments:",
                get_args_list(config.extra_java_args.as_deref(), |msg| {
                    LauncherSettingsMessage::GlobalJavaArgs(msg).into()
                }),
                widget::Space::with_height(5),
                "Global Pre-Launch Prefix:",
                widget::text(PREFIX_EXPLANATION).size(12).style(tsubtitle),
                get_args_list(
                    config
                        .global_settings
                        .as_ref()
                        .and_then(|n| n.pre_launch_prefix.as_deref()),
                    |n| LauncherSettingsMessage::GlobalPreLaunchPrefix(n).into(),
                ),
                args_split_by_space(menu.arg_split_by_space),
                widget::horizontal_rule(1),
                widget::row![
                    button_with_icon(icons::bin(), "Clear Java installs", 16)
                        .on_press(LauncherSettingsMessage::ClearJavaInstalls.into()),
                    widget::text(
                        "Might fix some Java problems.\nPerfectly safe, will be redownloaded."
                    )
                    .style(tsubtitle)
                    .size(12),
                ]
                .spacing(10)
                .wrap(),
            ]
            .spacing(SETTINGS_SPACING)
            .padding(16)
            .into(),
            LauncherSettingsTab::About => view_about_tab(),
        }
    }
}

fn view_portable_mode_section(
    menu: &MenuLauncherSettings,
) -> iced::Element<'static, Message, super::LauncherTheme> {
    let status = &menu.portable_mode_status.portable;
    let temp_path = &menu.temp_paths.portable;

    let t = |s| widget::text(s).size(12).style(tsubtitle);

    let is_active = status.is_some();

    let toggle_msg = if is_active {
        LauncherSettingsMessage::DisablePortableMode
    } else {
        LauncherSettingsMessage::EnablePortableMode
    };

    let portable_checkbox = widget::checkbox("Enable Portable Mode", is_active)
        .size(16)
        .text_size(15)
        .on_toggle(move |_| Message::LauncherSettings(toggle_msg.clone()));

    let mut col = widget::column![
        widget::row![
            widget::text("Portable Mode").size(16),
            widget::horizontal_space(),
            widget::text(if is_active { "ACTIVE" } else { "INACTIVE" })
                .size(12)
                .style(if is_active {
                    |t: &super::LauncherTheme| widget::text::Style {
                        color: Some(iced::Color::from_rgb8(0x94, 0xe2, 0xd5)),
                    }
                } else {
                    |t: &super::LauncherTheme| widget::text::Style {
                        color: Some(iced::Color::from_rgb8(0xf3, 0x8b, 0xa8)),
                    }
                }),
        ]
        .align_y(Alignment::Center),
        t("Store launcher data alongside the executable. (Highest Priority)"),
        widget::Space::with_height(5),
        portable_checkbox,
    ]
    .spacing(5);

    if is_active {
        let current_path = match status {
            Some(Some(p)) => p.to_string_lossy().into_owned(),
            _ => String::new(),
        };

        let has_changes = temp_path != &current_path;

        let mut path_row = widget::row![
            widget::text_input(
                "Leave blank to store right next to the executable.",
                temp_path
            )
            .on_input(
                |s| Message::LauncherSettings(LauncherSettingsMessage::SetTempPath(crate::state::PathKind::Portable, s))
            )
            .padding(6)
            .size(13)
            .width(Length::FillPortion(3)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        if has_changes {
            path_row = path_row.push(
                widget::button(widget::text("Apply Path").size(13))
                    .padding([5, 15])
                    .on_press(Message::LauncherSettings(
                        LauncherSettingsMessage::EnablePortableMode,
                    )),
            );
        }

        let custom_path_section = widget::column![
            widget::text("Custom Storage Path (Optional)").size(14),
            t("Specify a sub-directory path relative to the executable, or an absolute path."),
            widget::Space::with_height(2),
            path_row,
        ]
        .spacing(5)
        .padding(iced::Padding {
            top: 10.0,
            right: 0.0,
            bottom: 0.0,
            left: 25.0,
        });

        col = col.push(custom_path_section);
    }

    col.into()
}

fn view_system_redirect_section(
    menu: &MenuLauncherSettings,
) -> iced::Element<'static, Message, super::LauncherTheme> {
    let status = &menu.portable_mode_status.system_redirect;
    let portable_active = menu.portable_mode_status.portable.is_some();
    let temp_path = &menu.temp_paths.system_redirect;

    let t = |s| widget::text(s).size(12).style(tsubtitle);

    let is_active = status.is_some();

    let toggle_msg = if is_active {
        LauncherSettingsMessage::DisableSystemRedirect
    } else {
        LauncherSettingsMessage::EnableSystemRedirect
    };

    let redirect_checkbox = widget::checkbox("Enable System-Wide Redirection", is_active)
        .size(16)
        .text_size(15)
        .on_toggle(move |_| Message::LauncherSettings(toggle_msg.clone()));

    let mut status_row = widget::row![
        widget::text("System Redirection").size(16),
        widget::horizontal_space(),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    if is_active && portable_active {
        status_row = status_row.push(widget::text("OVERRIDDEN").size(12).style(
            |_: &super::LauncherTheme| widget::text::Style {
                color: Some(iced::Color::from_rgb8(0xfa, 0xa3, 0x56)),
            },
        ));
    }

    status_row = status_row.push(
        widget::text(if is_active { "ACTIVE" } else { "INACTIVE" })
            .size(12)
            .style(if is_active {
                |t: &super::LauncherTheme| widget::text::Style {
                    color: Some(iced::Color::from_rgb8(0x94, 0xe2, 0xd5)),
                }
            } else {
                |t: &super::LauncherTheme| widget::text::Style {
                    color: Some(iced::Color::from_rgb8(0xf3, 0x8b, 0xa8)),
                }
            }),
    );

    let mut col = widget::column![
        status_row,
        t("Redirect data globally via the system data directory (~/.local/share/QuantumLauncher)."),
    ]
    .spacing(5);

    if is_active && portable_active {
        col = col.push(
            t("Warning: Portable Mode is currently active and takes priority over this setting.")
                .style(|_: &super::LauncherTheme| widget::text::Style {
                    color: Some(iced::Color::from_rgb8(0xfa, 0xa3, 0x56)),
                }),
        );
    }

    col = col.push(widget::Space::with_height(5));
    col = col.push(redirect_checkbox);

    if is_active {
        let current_path = match status {
            Some(Some(p)) => p.to_string_lossy().into_owned(),
            _ => String::new(),
        };

        let has_changes = temp_path != &current_path;

        let mut path_row = widget::row![
            widget::text_input("Enter redirection path...", temp_path)
                .on_input(|s| Message::LauncherSettings(
                    LauncherSettingsMessage::SetTempPath(crate::state::PathKind::SystemRedirect, s)
                ))
                .padding(6)
                .size(13)
                .width(Length::FillPortion(3)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        if has_changes {
            path_row = path_row.push(
                widget::button(widget::text("Apply Path").size(13))
                    .padding([5, 15])
                    .on_press(Message::LauncherSettings(
                        LauncherSettingsMessage::EnableSystemRedirect,
                    )),
            );
        }

        let custom_path_section = widget::column![
            widget::text("Global Redirect Path").size(14),
            t("All instances will be redirected to this location unless a local portable override exists."),
            widget::Space::with_height(2),
            path_row,
        ]
        .spacing(5)
        .padding(iced::Padding {
            top: 10.0,
            right: 0.0,
            bottom: 0.0,
            left: 25.0,
        });

        col = col.push(custom_path_section);
    }

    col.into()
}

fn view_about_tab() -> Element<'static> {
    let gpl3_button = widget::button(underline(
        widget::text("GNU GPLv3 License").size(12),
        Color::Light,
    ))
    .padding(0)
    .style(|n: &LauncherTheme, status| n.style_button(status, StyleButton::FlatDark))
    .on_press(Message::LicenseChangeTab(crate::state::LicenseTab::Gpl3));

    let links = widget::row![
        button_with_icon(icons::globe(), "Website", 16)
            .on_press(Message::CoreOpenLink(WEBSITE.to_owned())),
        button_with_icon(icons::github(), "Github", 16)
            .on_press(Message::CoreOpenLink(GITHUB.to_owned())),
        button_with_icon(icons::discord(), "Discord", 16)
            .on_press(Message::CoreOpenLink(DISCORD.to_owned())),
    ]
    .spacing(5)
    .wrap();

    let menus = widget::row![
        widget::button("Changelog").on_press(Message::CoreOpenChangeLog),
        widget::button("Welcome Screen").on_press(Message::CoreOpenIntro),
        widget::button("Licenses").on_press(Message::LicenseOpen),
    ]
    .spacing(5)
    .wrap();

    widget::column![
        widget::column![
            widget::text("About QuantumLauncher").size(20),
            "Copyright 2025 Mrmayman & Contributors"
        ]
        .spacing(5),
        menus,
        links,
        widget::button(widget::image(IMG_ICED.clone()).height(40))
            .on_press(Message::CoreOpenLink("https://iced.rs".to_owned()))
            .padding(5)
            .style(|n: &LauncherTheme, status| n.style_button(status, StyleButton::Flat)),
        widget::horizontal_rule(1),
        widget::column![
            widget::row![
                widget::text("QuantumLauncher is free and open source software under the ")
                    .size(12),
                gpl3_button,
            ]
            .wrap(),
            widget::text(
                r"No warranty is provided for this software.
You're free to share, modify, and redistribute it under the same license."
            )
            .size(12),
            widget::text(
                r"If you like this launcher, consider sharing it with your friends.
Every new user motivates me to keep working on this :)"
            )
            .size(12),
        ]
        .padding(iced::Padding {
            top: 10.0,
            bottom: 10.0,
            left: 15.0,
            right: 10.0,
        })
        .spacing(5),
    ]
    .padding(16)
    .spacing(SETTINGS_SPACING)
    .into()
}
