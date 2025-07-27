use std::time::{Duration, Instant};

use auth::AccountData;
use iced::Task;
use ql_core::IntoStringError;
use ql_instances::auth;

use crate::{
    config::ConfigAccount,
    state::{
        AccountMessage, Launcher, LittleSkinOauth, MenuLoginAlternate, MenuLoginMS, Message,
        ProgressBar, State, NEW_ACCOUNT_NAME, OFFLINE_ACCOUNT_NAME,
    },
};

impl Launcher {
    pub fn update_account(&mut self, msg: AccountMessage) -> Task<Message> {
        match msg {
            AccountMessage::Response1 { r: Err(err), .. }
            | AccountMessage::Response2(Err(err))
            | AccountMessage::Response3(Err(err))
            | AccountMessage::AltLoginResponse(Err(err))
            | AccountMessage::RefreshComplete(Err(err)) => {
                self.set_error(err);
            }
            AccountMessage::Selected(account) => {
                return self.account_selected(account);
            }
            AccountMessage::Response1 {
                r: Ok(code),
                is_from_welcome_screen,
            } => {
                return self.account_response_1(code, is_from_welcome_screen);
            }
            AccountMessage::Response2(Ok(token)) => {
                return self.account_response_2(token);
            }
            AccountMessage::Response3(Ok(data)) => {
                return self.account_response_3(data);
            }
            AccountMessage::LogoutCheck => {
                let username = self.accounts_selected.as_ref().unwrap();
                self.state = State::ConfirmAction {
                    msg1: format!("log out of your account: {username}"),
                    msg2: "You can always log in later".to_owned(),
                    yes: Message::Account(AccountMessage::LogoutConfirm),
                    no: Message::LaunchScreenOpen {
                        message: None,
                        clear_selection: false,
                    },
                }
            }
            AccountMessage::LittleSkinDeviceCodeRequested => {
                todo!("Handle LittleSkinDeviceCodeRequested");
            }
            AccountMessage::LittleSkinDeviceCodeReady {
                user_code,
                verification_uri,
                expires_in,
                interval,
                device_code,
            } => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.oauth = Some(LittleSkinOauth {
                        // device_code: device_code.clone(),
                        user_code: user_code.clone(),
                        verification_uri: verification_uri.clone(),
                        device_code_expires_at: Instant::now() + Duration::from_secs(expires_in),
                    });
                    menu.is_loading = false;
                }

                // Start polling for token
                let device_code_clone = device_code.clone();
                return Task::perform(
                    ql_instances::auth::littleskin::oauth::poll_device_token(
                        device_code_clone,
                        interval,
                        expires_in,
                    ),
                    |resp| match resp {
                        Ok(account) => {
                            Message::Account(AccountMessage::AltLoginResponse(Ok(account)))
                        }
                        Err(e) => Message::Account(AccountMessage::LittleSkinDeviceCodeError(
                            e.to_string(),
                        )),
                    },
                );
            }
            AccountMessage::LittleSkinDeviceCodeError(err_msg) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.is_loading = false;
                    menu.device_code_error = Some(err_msg);
                }
            }
            AccountMessage::LittleSkinDeviceCodePollResult(_) => {
                todo!("Handle LittleSkinDeviceCodePollResult");
            }
            AccountMessage::LogoutConfirm => {
                let username = self.accounts_selected.clone().unwrap();
                let account_type = self
                    .accounts
                    .get(&username)
                    .map(|n| n.account_type)
                    .unwrap_or(auth::AccountType::Microsoft);

                if let Err(err) = match account_type {
                    auth::AccountType::Microsoft => auth::ms::logout(&username),
                    auth::AccountType::ElyBy => {
                        auth::elyby::logout(username.strip_suffix(" (elyby)").unwrap_or(&username))
                    }
                    auth::AccountType::LittleSkin => auth::littleskin::logout(&username),
                } {
                    self.set_error(err);
                }
                if let Some(accounts) = &mut self.config.accounts {
                    accounts.remove(&username);
                }
                self.accounts.remove(&username);
                if let Some(idx) = self
                    .accounts_dropdown
                    .iter()
                    .enumerate()
                    .find_map(|(i, n)| (*n == username).then_some(i))
                {
                    self.accounts_dropdown.remove(idx);
                }
                let selected_account = self
                    .accounts_dropdown
                    .first()
                    .cloned()
                    .unwrap_or_else(|| OFFLINE_ACCOUNT_NAME.to_owned());
                self.accounts_selected = Some(selected_account);

                return self.go_to_launch_screen(Option::<String>::None);
            }
            AccountMessage::RefreshComplete(Ok(data)) => {
                self.accounts.insert(data.get_username_modified(), data);

                let account_data = if let Some(account) = &self.accounts_selected {
                    if account == NEW_ACCOUNT_NAME || account == OFFLINE_ACCOUNT_NAME {
                        None
                    } else {
                        self.accounts.get(account).cloned()
                    }
                } else {
                    None
                };

                return Task::batch([
                    self.go_to_launch_screen::<String>(None),
                    self.launch_game(account_data),
                ]);
            }

            AccountMessage::OpenMicrosoft {
                is_from_welcome_screen,
            } => {
                self.state = State::GenericMessage("Loading Login...".to_owned());
                return Task::perform(auth::ms::login_1_link(), move |n| {
                    Message::Account(AccountMessage::Response1 {
                        r: n.strerr(),
                        is_from_welcome_screen,
                    })
                });
            }
            AccountMessage::OpenElyBy {
                is_from_welcome_screen,
            } => {
                self.state = State::LoginAlternate(MenuLoginAlternate {
                    username: String::new(),
                    password: String::new(),
                    is_loading: false,
                    otp: None,
                    show_password: false,
                    is_from_welcome_screen,

                    is_littleskin: false,
                    device_code_error: None,
                    oauth: None,
                });
            }

            AccountMessage::AltUsernameInput(username) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.username = username;
                }
            }
            AccountMessage::AltPasswordInput(password) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.password = password;
                }
            }
            AccountMessage::AltOtpInput(otp) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.otp = Some(otp);
                }
            }
            AccountMessage::AltShowPassword(t) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.show_password = t;
                }
            }

            AccountMessage::AltLogin => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    let mut password = menu.password.clone();
                    if let Some(otp) = &menu.otp {
                        password.push(':');
                        password.push_str(otp);
                    }
                    menu.is_loading = true;

                    if menu.is_littleskin {
                        return Task::perform(
                            auth::littleskin::login_new(menu.username.clone(), password),
                            |n| Message::Account(AccountMessage::AltLoginResponse(n.strerr())),
                        );
                    } else {
                        return Task::perform(
                            auth::elyby::login_new(menu.username.clone(), password),
                            |n| Message::Account(AccountMessage::AltLoginResponse(n.strerr())),
                        );
                    }
                }
            }
            AccountMessage::AltLoginResponse(Ok(acc)) => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.is_loading = false;
                    match acc {
                        auth::elyby::Account::Account(data) => {
                            return self.account_response_3(data);
                        }
                        auth::elyby::Account::NeedsOTP => {
                            menu.otp = Some(String::new());
                        }
                    }
                }
            }
            AccountMessage::OpenLittleSkin {
                is_from_welcome_screen,
            } => {
                self.state = State::LoginAlternate(MenuLoginAlternate {
                    username: String::new(),
                    password: String::new(),
                    is_loading: false,
                    otp: None,
                    show_password: false,
                    is_from_welcome_screen,
                    oauth: None,
                    device_code_error: None,
                    is_littleskin: true,
                });
            }

            AccountMessage::LittleSkinOauthButtonClicked => {
                if let State::LoginAlternate(menu) = &mut self.state {
                    menu.is_loading = true;
                }

                return Task::perform(auth::littleskin::oauth::request_device_code(), |resp| {
                    Message::Account(match resp {
                        Ok(code) => AccountMessage::LittleSkinDeviceCodeReady {
                            user_code: code.user_code,
                            verification_uri: code.verification_uri,
                            expires_in: code.expires_in,
                            interval: code.interval,
                            device_code: code.device_code,
                        },
                        Err(e) => AccountMessage::LittleSkinDeviceCodeError(e.to_string()),
                    })
                });
            }
        }
        Task::none()
    }

    fn account_selected(&mut self, account: String) -> Task<Message> {
        if account == NEW_ACCOUNT_NAME {
            self.state = State::AccountLogin;
        } else {
            self.accounts_selected = Some(account);
        }
        Task::none()
    }

    pub fn account_refresh(&mut self, account: &AccountData) -> Task<Message> {
        match account.account_type {
            auth::AccountType::Microsoft => {
                let (sender, receiver) = std::sync::mpsc::channel();
                self.state = State::AccountLoginProgress(ProgressBar::with_recv(receiver));

                Task::perform(
                    auth::ms::login_refresh(
                        account.username.clone(),
                        account.refresh_token.clone(),
                        Some(sender),
                    ),
                    |n| Message::Account(AccountMessage::RefreshComplete(n.strerr())),
                )
            }
            auth::AccountType::ElyBy => Task::perform(
                auth::elyby::login_refresh(account.username.clone(), account.refresh_token.clone()),
                |n| Message::Account(AccountMessage::RefreshComplete(n.strerr())),
            ),
            auth::AccountType::LittleSkin => Task::perform(
                auth::littleskin::login_refresh(
                    account.username.clone(),
                    account.refresh_token.clone(),
                ),
                |n| Message::Account(AccountMessage::RefreshComplete(n.strerr())),
            ),
        }
    }

    fn account_response_3(&mut self, data: AccountData) -> Task<Message> {
        self.accounts_dropdown.insert(0, data.username.clone());

        let accounts = self.config.accounts.get_or_insert_default();
        let username = data.get_username_modified();
        accounts.insert(
            username.clone(),
            ConfigAccount {
                uuid: data.uuid.clone(),
                skin: None,
                account_type: Some(data.account_type.to_string()),
                username_nice: Some(data.nice_username.clone()),
            },
        );

        self.accounts_selected = Some(username.clone());
        self.accounts.insert(username.clone(), data);

        self.go_to_launch_screen::<String>(None)
    }

    fn account_response_2(&mut self, token: auth::ms::AuthTokenResponse) -> Task<Message> {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.state = State::AccountLoginProgress(ProgressBar::with_recv(receiver));
        Task::perform(auth::ms::login_3_xbox(token, Some(sender), true), |n| {
            Message::Account(AccountMessage::Response3(n.strerr()))
        })
    }

    fn account_response_1(
        &mut self,
        code: auth::ms::AuthCodeResponse,
        is_from_welcome_screen: bool,
    ) -> Task<Message> {
        let (task, handle) = Task::perform(auth::ms::login_2_wait(code.clone()), |n| {
            Message::Account(AccountMessage::Response2(n.strerr()))
        })
        .abortable();

        self.state = State::LoginMS(MenuLoginMS {
            url: code.verification_uri,
            code: code.user_code,
            is_from_welcome_screen,
            _cancel_handle: handle.abort_on_drop(),
        });

        task
    }
}
