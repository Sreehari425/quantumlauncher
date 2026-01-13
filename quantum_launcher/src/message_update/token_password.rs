//! Handler for encrypted token storage password prompt.

use iced::Task;
use ql_instances::auth::{encrypted_store, token_store::TokenStorageMethod};

use crate::state::{Launcher, MenuLaunch, Message, State, TokenPasswordMessage};

impl Launcher {
    pub fn update_token_password(&mut self, msg: TokenPasswordMessage) -> Task<Message> {
        match msg {
            TokenPasswordMessage::PasswordInput(password) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.password = password;
                }
            }

            TokenPasswordMessage::PasswordConfirmInput(password) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.password_confirm = password;
                }
            }

            TokenPasswordMessage::ShowPassword(show) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.show_password = show;
                }
            }

            TokenPasswordMessage::Submit => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    let password = menu.password.clone();
                    let is_existing = menu.is_existing;

                    menu.is_loading = true;
                    menu.error = None;

                    if is_existing {
                        // Try to unlock existing store
                        return Task::perform(
                            async move { encrypted_store::unlock(&password).map_err(|e| e.to_string()) },
                            |result| {
                                Message::TokenPassword(TokenPasswordMessage::UnlockResult(result))
                            },
                        );
                    } else {
                        // Create new store with the password
                        let password_confirm = menu.password_confirm.clone();
                        if password != password_confirm {
                            menu.error = Some("Passwords do not match".to_owned());
                            menu.is_loading = false;
                            return Task::none();
                        }

                        return Task::perform(
                            async move {
                                encrypted_store::initialize_new(&password)
                                    .map_err(|e| e.to_string())
                            },
                            |result| {
                                Message::TokenPassword(TokenPasswordMessage::UnlockResult(result))
                            },
                        );
                    }
                }
            }

            TokenPasswordMessage::UnlockResult(result) => {
                if let State::TokenPasswordPrompt(menu) = &mut self.state {
                    menu.is_loading = false;

                    match result {
                        Ok(()) => {
                            // Successfully unlocked/created - set global storage method
                            TokenStorageMethod::set_global(TokenStorageMethod::EncryptedFile);

                            // Now load accounts
                            let (accounts, accounts_dropdown, selected_account) =
                                crate::state::load_accounts(&mut self.config);

                            self.accounts = accounts;
                            self.accounts_dropdown = accounts_dropdown;
                            self.accounts_selected = Some(selected_account);

                            // Go to the normal launch screen
                            let mut launch = MenuLaunch::default();
                            launch.resize_sidebar(crate::config::SIDEBAR_WIDTH);
                            self.state = State::Launch(launch);
                        }
                        Err(err) => {
                            menu.error = Some(err);
                        }
                    }
                }
            }

            TokenPasswordMessage::Skip => {
                // User skipped unlock at startup - continue in offline mode without unlocking
                // Keep encrypted file setting, just don't unlock for this session
                // (tokens won't be accessible, but user can still play offline)
                
                let mut launch = MenuLaunch::default();
                launch.resize_sidebar(crate::config::SIDEBAR_WIDTH);
                self.state = State::Launch(launch);
            }

            TokenPasswordMessage::Cancel => {
                // User cancelled new store creation from settings - revert to keyring
                self.config.token_storage = Some(crate::config::TokenStorageMethod::Keyring);
                TokenStorageMethod::set_global(TokenStorageMethod::Keyring);

                self.go_to_launcher_settings();
                if let State::LauncherSettings(menu) = &mut self.state {
                    menu.selected_tab = crate::state::LauncherSettingsTab::Security;
                }
            }
        }
        Task::none()
    }
}
