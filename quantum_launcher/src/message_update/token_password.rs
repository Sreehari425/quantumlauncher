//! Handler for `TokenPasswordMessage` — password prompt for the encrypted token store.

use iced::Task;
use ql_instances::auth::encrypted_store;

use crate::state::{
    Launcher, Message, State, TokenPasswordMessage,
};

impl Launcher {
    pub fn update_token_password(&mut self, msg: TokenPasswordMessage) -> Task<Message> {
        match msg {
            TokenPasswordMessage::PasswordChanged(p) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.password = p;
                    menu.error = None;
                }
            }
            TokenPasswordMessage::ConfirmPasswordChanged(p) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    if let Some(confirm) = &mut menu.confirm_password {
                        *confirm = p;
                    }
                    menu.error = None;
                }
            }
            TokenPasswordMessage::ToggleShowPassword(show) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.show_password = show;
                }
            }
            TokenPasswordMessage::Submit => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.is_loading = true;
                    let password = menu.password.clone();
                    let confirm = menu.confirm_password.clone();

                    if let Some(confirm_pw) = confirm {
                        // "Create new store" flow
                        if password != confirm_pw {
                            menu.is_loading = false;
                            menu.error = Some("Passwords do not match.".to_owned());
                            return Task::none();
                        }
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    encrypted_store::initialize_new(&password)
                                        .map_err(|e| e.to_string())
                                })
                                .await
                                .map_err(|e| e.to_string())
                                .and_then(|r| r)
                            },
                            |res| Message::TokenPassword(TokenPasswordMessage::SubmitDone(res)),
                        );
                    } else {
                        // "Unlock existing store" flow
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    encrypted_store::unlock(&password).map_err(|e| e.to_string())
                                })
                                .await
                                .map_err(|e| e.to_string())
                                .and_then(|r| r)
                            },
                            |res| Message::TokenPassword(TokenPasswordMessage::SubmitDone(res)),
                        );
                    }
                }
            }
            TokenPasswordMessage::SubmitDone(result) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.is_loading = false;
                    match result {
                        Ok(()) => {
                            // Unlock succeeded — reload accounts that were skipped at startup
                            let new_accounts = crate::state::reload_encrypted_accounts(&mut self.config);
                            self.accounts.extend(new_accounts.0);
                            for entry in new_accounts.1 {
                                if !self.accounts_dropdown.contains(&entry) {
                                    self.accounts_dropdown.insert(0, entry);
                                }
                            }
                            return self.go_to_launch_screen::<&str>(None);
                        }
                        Err(err) => {
                            menu.error = Some(err);
                        }
                    }
                }
            }
            TokenPasswordMessage::Skip => {
                return self.go_to_launch_screen::<&str>(None);
            }
        }
        Task::none()
    }
}
