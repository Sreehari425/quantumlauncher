use iced::widget;

use crate::{
    config::LauncherConfig,
    icon_manager,
    menu_renderer::{
        button_with_icon, center_x, get_color_schemes, get_theme_selector, Element, DISCORD,
    },
    state::{AccountMessage, MenuWelcome, Message},
};

use super::IMG_LOGO;

impl MenuWelcome {
    pub fn view<'a>(&'a self, config: &'a LauncherConfig) -> Element<'a> {
        match self {
            MenuWelcome::P1InitialScreen => widget::column![
                widget::vertical_space(),
                center_x(widget::image(IMG_LOGO.clone()).width(200)),
                center_x(widget::text("Welcome to QuantumLauncher!").size(20)),
                center_x(widget::button("Get Started").on_press(Message::WelcomeContinueToTheme)),
                widget::vertical_space(),
            ]
            .align_x(iced::alignment::Horizontal::Center)
            .spacing(10)
            .into(),
            MenuWelcome::P2Theme => {
                let style = get_color_schemes(config);
                let (light, dark) = get_theme_selector(config);
                widget::column![
                    widget::vertical_space(),
                    center_x(widget::text("Customize your launcher!").size(24)),
                    widget::row![
                        widget::horizontal_space(),
                        "Select Theme:",
                        widget::row![light, dark].spacing(5),
                        widget::horizontal_space(),
                    ]
                    .spacing(10),
                    widget::row![
                        widget::horizontal_space(),
                        "Select Color Scheme:",
                        style,
                        widget::horizontal_space(),
                    ]
                    .spacing(10),
                    widget::Space::with_height(5),
                    center_x("Oh, and also..."),
                    center_x(
                        button_with_icon(icon_manager::chat(), "Join our Discord", 16)
                            .on_press(Message::CoreOpenLink(DISCORD.to_owned()))
                    ),
                    widget::Space::with_height(5),
                    center_x(widget::button("Continue").on_press(Message::WelcomeContinueToAuth)),
                    widget::vertical_space(),
                ]
                .spacing(10)
                .into()
            }
            MenuWelcome::P3Auth => widget::column![
                widget::vertical_space(),
                center_x(
                    widget::button("Login to Microsoft").on_press(Message::Account(
                        AccountMessage::OpenMicrosoft {
                            is_from_welcome_screen: true
                        }
                    ))
                ),
                center_x(widget::button("Login to ely.by").on_press(Message::Account(
                    AccountMessage::OpenElyBy {
                        is_from_welcome_screen: true
                    }
                ))),
                center_x(
                    widget::button("Login to littleskin").on_press(Message::Account(
                        AccountMessage::OpenLittleSkin {
                            is_from_welcome_screen: true
                        }
                    ))
                ),
                widget::Space::with_height(7),
                center_x(widget::text("OR").size(20)),
                widget::Space::with_height(7),
                center_x(
                    widget::text_input("Enter username...", &config.username)
                        .width(200)
                        .on_input(Message::LaunchUsernameSet)
                ),
                center_x(
                    widget::button(center_x("Continue"))
                        .width(200)
                        .on_press_maybe((!config.username.is_empty()).then_some(
                            Message::LaunchScreenOpen {
                                message: None,
                                clear_selection: true,
                                is_server: Some(false)
                            }
                        ))
                ),
                widget::vertical_space(),
            ]
            .spacing(5)
            .into(),
        }
    }
}
