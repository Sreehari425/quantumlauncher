use iced::widget;

use crate::{
    icon_manager,
    state::{AccountMessage, MenuLoginElyBy, MenuLoginLittleSkin, MenuLoginMS, Message, NEW_ACCOUNT_NAME},
};

use super::{back_button, button_with_icon, Element};

impl MenuLoginElyBy {
    pub fn view(&self, tick_timer: usize) -> Element {
        let status: Element = if self.is_loading {
            let dots = ".".repeat((tick_timer % 3) + 1);
            widget::text!("Loading{dots}").into()
        } else {
            button_with_icon(icon_manager::tick(), "Login", 16)
                .on_press(Message::Account(AccountMessage::ElyByLogin))
                .into()
        };

        let padding = iced::Padding {
            top: 5.0,
            bottom: 5.0,
            right: 10.0,
            left: 10.0,
        };

        let password_input = widget::text_input("Enter Password...", &self.password)
            .padding(padding)
            .on_input(|n| Message::Account(AccountMessage::ElyByPasswordInput(n)));
        let password_input = if self.password.is_empty() || self.show_password {
            password_input
        } else {
            password_input.font(iced::Font::with_name("Password Asterisks"))
        };

        widget::column![
            back_button().on_press(if self.is_from_welcome_screen {
                Message::WelcomeContinueToAuth
            } else {
                Message::Account(AccountMessage::Selected(NEW_ACCOUNT_NAME.to_owned()))
            }),
            widget::row![
                widget::horizontal_space(),
                widget::column![
                    widget::vertical_space(),
                    widget::text("Username/Email:").size(12),
                    widget::text_input("Enter Username/Email...", &self.username)
                        .padding(padding)
                        .on_input(|n| Message::Account(AccountMessage::ElyByUsernameInput(n))),
                    widget::text("Password:").size(12),
                    password_input,
                    widget::checkbox("Show Password", self.show_password)
                        .size(14)
                        .text_size(14)
                        .on_toggle(|t| Message::Account(AccountMessage::ElyByShowPassword(t))),
                    widget::Column::new().push_maybe(self.otp.as_deref().map(|otp| {
                        widget::column![
                            widget::text("OTP:").size(12),
                            widget::text_input("Enter Username/Email...", otp)
                                .padding(padding)
                                .on_input(|n| Message::Account(AccountMessage::ElyByOtpInput(n))),
                        ]
                        .spacing(5)
                    })),
                    status,
                    widget::Space::with_height(5),
                    widget::row![
                        widget::text("Or").size(14),
                        widget::button(widget::text("Create an account").size(14)).on_press(
                            Message::CoreOpenLink("https://account.ely.by/register".to_owned())
                        )
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(5)
                    .wrap(),
                    widget::vertical_space(),
                ]
                .align_x(iced::Alignment::Center)
                .spacing(5),
                widget::horizontal_space(),
            ]
        ]
        .padding(10)
        .into()
    }
}

impl MenuLoginLittleSkin {
    pub fn view(&self, tick_timer: usize) -> Element {
        let padding = iced::Padding {
            top: 5.0,
            bottom: 5.0,
            right: 10.0,
            left: 10.0,
        };

        let password_input = widget::text_input("Enter Password...", &self.password)
            .padding(padding)
            .on_input(|n| Message::Account(AccountMessage::LittleSkinPasswordInput(n)));
        let password_input = if self.password.is_empty() || self.show_password {
            password_input
        } else {
            password_input.font(iced::Font::with_name("Password Asterisks"))
        };

        let oauth_test_button = widget::button("Login with OAuth")
            .on_press(Message::Account(AccountMessage::OauthTestButtonClicked));

        // Show error if present
        let error_msg = self.device_code_error.as_ref().map(|err| {
            use crate::stylesheet::color::Color;
            widget::text(err)
                .size(14)
        });

        // Device code flow UI
        let device_code_flow = if self.device_code_polling {
            let time_left = self.device_code_expires_at
                .map(|instant| {
                    let now = std::time::Instant::now();
                    if instant > now {
                        (instant - now).as_secs()
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            let code = self.user_code.as_deref().unwrap_or("");
            let url = self.verification_uri.as_deref().unwrap_or("");
            let code_row = widget::row![
                widget::text!("Code: {code}").size(18),
                widget::button("Copy").on_press(Message::CoreCopyText(code.to_owned())),
            ].spacing(10);
            let url_row = widget::row![
                widget::text!("Link: {url}").size(14),
                widget::button("Open").on_press(Message::CoreOpenLink(url.to_owned())),
            ].spacing(10);
            widget::column![
                widget::vertical_space(),
                widget::text("LittleSkin Device Login").size(20),
                widget::text("Open this link and enter the code:").size(14),
                code_row,
                url_row,
                widget::text!("Expires in: {}s", time_left).size(12),
                widget::vertical_space(),
                widget::text("Waiting for login...").size(14),
                widget::vertical_space(),
            ].push_maybe(error_msg)
            .spacing(5).align_x(iced::Alignment::Center)
        } else {
            // Show normal login form
            let status: Element = if self.is_loading {
                let dots = ".".repeat((tick_timer % 3) + 1);
                widget::text!("Loading{dots}").into()
            } else {
                button_with_icon(icon_manager::tick(), "Login", 16)
                    .on_press(Message::Account(AccountMessage::LittleSkinLogin))
                    .into()
            };
            widget::column![
                widget::vertical_space(),
                widget::text("Username/Email:").size(12),
                widget::text_input("Enter Username/Email...", &self.username)
                    .padding(padding)
                    .on_input(|n| Message::Account(AccountMessage::LittleSkinUsernameInput(n))),
                widget::text("Password:").size(12),
                password_input,
                widget::checkbox("Show Password", self.show_password)
                    .size(14)
                    .text_size(14)
                    .on_toggle(|t| Message::Account(AccountMessage::LittleSkinShowPassword(t))),
                widget::Column::new().push_maybe(self.otp.as_deref().map(|otp| {
                    widget::column![
                        widget::text("OTP:").size(12),
                        widget::text_input("Enter Username/Email...", otp)
                            .padding(padding)
                            .on_input(|n| Message::Account(AccountMessage::LittleSkinOtpInput(n))),
                    ]
                    .spacing(5)
                })),
                status,
                oauth_test_button,
                widget::Space::with_height(5),
                widget::row![
                    widget::text("Or").size(14),
                    widget::button(widget::text("Create an account").size(14)).on_press(
                        Message::CoreOpenLink("https://littleskin.cn/auth/register".to_owned())
                    )
                ]
                .align_y(iced::Alignment::Center)
                .spacing(5)
                .wrap(),
                widget::vertical_space(),
            ].push_maybe(error_msg)
            .align_x(iced::Alignment::Center)
            .spacing(5)
        };

        widget::column![
            back_button().on_press(if self.is_from_welcome_screen {
                Message::WelcomeContinueToAuth
            } else {
                Message::Account(AccountMessage::Selected(NEW_ACCOUNT_NAME.to_owned()))
            }),
            widget::row![
                widget::horizontal_space(),
                device_code_flow,
                widget::horizontal_space(),
            ]
        ]
        .padding(10)
        .into()
    }
}


impl MenuLoginMS {
    pub fn view<'a>(&self) -> Element<'a> {
        widget::column![
            back_button().on_press(if self.is_from_welcome_screen {
                Message::WelcomeContinueToAuth
            } else {
                Message::Account(AccountMessage::Selected(NEW_ACCOUNT_NAME.to_owned()))
            }),
            widget::row!(
                widget::horizontal_space(),
                widget::column!(
                    widget::vertical_space(),
                    widget::text("Login to Microsoft").size(20),
                    "Open this link and enter the code:",
                    widget::text!("Code: {}", self.code),
                    widget::button("Copy").on_press(Message::CoreCopyText(self.code.clone())),
                    widget::text!("Link: {}", self.url),
                    widget::button("Open").on_press(Message::CoreOpenLink(self.url.clone())),
                    widget::vertical_space(),
                )
                .spacing(5)
                .align_x(iced::Alignment::Center),
                widget::horizontal_space()
            )
        ]
        .padding(10)
        .into()
    }
}
