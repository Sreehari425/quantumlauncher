// QuantumLauncher TUI - Application State

use std::{error::Error, fmt};
use ql_core::ListEntry;
use serde_json::Value;
use crate::config::{LauncherConfig, ConfigAccount};

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TabId {
    Instances,
    Create,
    Settings,
    Accounts,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub account_type: String,
    pub uuid: String,
    pub is_logged_in: bool,
}

impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TabId::Instances => write!(f, "Instances"),
            TabId::Create => write!(f, "Create"),
            TabId::Settings => write!(f, "Settings"),
            TabId::Accounts => write!(f, "Accounts"),
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub name: String,
    pub version: String,
    pub loader: String,
}

#[derive(Debug)]
pub struct App {
    pub current_tab: TabId,
    pub instances: Vec<Instance>,
    pub selected_instance: usize,
    pub available_versions: Vec<ListEntry>,
    pub selected_version: usize,
    pub new_instance_name: String,
    pub is_editing_name: bool,
    pub show_help_popup: bool,
    pub accounts: Vec<Account>,
    pub selected_account: usize,
    pub login_username: String,
    pub login_password: String,
    pub is_login_mode: bool,
    pub current_account: Option<String>,
    pub status_message: String,
    pub should_quit: bool,
    pub is_loading: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            current_tab: TabId::Instances,
            instances: Vec::new(),
            selected_instance: 0,
            available_versions: Vec::new(),
            selected_version: 0,
            new_instance_name: String::new(),
            is_editing_name: false,
            show_help_popup: false,
            accounts: Vec::new(),
            selected_account: 0,
            login_username: String::new(),
            login_password: String::new(),
            is_login_mode: false,
            current_account: None,
            status_message: "Welcome to QuantumLauncher TUI! Press '?' for help, 'q' to quit.".to_string(),
            should_quit: false,
            is_loading: false,
        };
        
        // Load instances and accounts on startup
        app.refresh();
        app.load_accounts();
        app
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabId::Instances => TabId::Create,
            TabId::Create => TabId::Settings,
            TabId::Settings => TabId::Accounts,
            TabId::Accounts => TabId::Instances,
        };
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabId::Instances => TabId::Accounts,
            TabId::Create => TabId::Instances,
            TabId::Settings => TabId::Create,
            TabId::Accounts => TabId::Settings,
        };
    }

    pub fn set_tab(&mut self, tab: TabId) {
        self.current_tab = tab;
    }

    pub fn next_item(&mut self) {
        match self.current_tab {
            TabId::Instances => {
                if !self.instances.is_empty() {
                    self.selected_instance = (self.selected_instance + 1) % self.instances.len();
                }
            }
            TabId::Create => {
                if !self.available_versions.is_empty() {
                    self.selected_version = (self.selected_version + 1) % self.available_versions.len();
                }
            }
            TabId::Accounts => {
                if !self.accounts.is_empty() {
                    self.selected_account = (self.selected_account + 1) % self.accounts.len();
                }
            }
            _ => {}
        }
    }

    pub fn prev_item(&mut self) {
        match self.current_tab {
            TabId::Instances => {
                if !self.instances.is_empty() && self.selected_instance > 0 {
                    self.selected_instance -= 1;
                } else if !self.instances.is_empty() {
                    self.selected_instance = self.instances.len() - 1;
                }
            }
            TabId::Create => {
                if !self.available_versions.is_empty() && self.selected_version > 0 {
                    self.selected_version -= 1;
                } else if !self.available_versions.is_empty() {
                    self.selected_version = self.available_versions.len() - 1;
                }
            }
            TabId::Accounts => {
                if !self.accounts.is_empty() && self.selected_account > 0 {
                    self.selected_account -= 1;
                } else if !self.accounts.is_empty() {
                    self.selected_account = self.accounts.len() - 1;
                }
            }
            _ => {}
        }
    }

    pub fn select_item(&mut self) {
        match self.current_tab {
            TabId::Instances => {
                if let Some(instance) = self.instances.get(self.selected_instance) {
                    self.status_message = format!("Selected instance: {} - Press Enter again to launch (feature coming soon)", instance.name);
                    // TODO: Implement instance launching
                    // For now, just show a message
                }
            }
            TabId::Create => {
                if !self.new_instance_name.is_empty() && !self.available_versions.is_empty() {
                    let version = &self.available_versions[self.selected_version];
                    self.status_message = format!("Creating instance '{}' with version '{}' (feature coming soon)", self.new_instance_name, version.name);
                    // TODO: Implement actual instance creation
                    // self.create_instance();
                } else if self.new_instance_name.is_empty() {
                    self.status_message = "Please enter an instance name first".to_string();
                } else {
                    self.status_message = "No version selected".to_string();
                }
            }
            _ => {}
        }
    }

    pub fn refresh(&mut self) {
        use std::path::PathBuf;
        use crate::state::get_entries;
        use ql_core::json::{InstanceConfigJson, VersionDetails};
        use ql_core::file_utils;

        self.is_loading = true;
        self.status_message = "Refreshing...".to_string();

        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(get_entries("instances".to_owned(), false)) {
                    Ok((instance_names, _)) => {
                        self.instances.clear();
                        let launcher_dir = match file_utils::get_launcher_dir() {
                            Ok(dir) => dir,
                            Err(_) => PathBuf::from(".config/QuantumLauncher"),
                        };
                        let instances_dir = launcher_dir.join("instances");
                        for name in instance_names {
                            let instance_dir = instances_dir.join(&name);
                            // Try to read config.json
                            let loader = match rt.block_on(InstanceConfigJson::read_from_dir(&instance_dir)) {
                                Ok(cfg) => cfg.mod_type,
                                Err(_) => "Vanilla".to_string(),
                            };
                            // Try to read details.json
                            let version = match rt.block_on(VersionDetails::load_from_path(&instance_dir)) {
                                Ok(details) => details.id,
                                Err(_) => "Unknown".to_string(),
                            };
                            self.instances.push(Instance {
                                name: name.clone(),
                                version,
                                loader,
                            });
                        }
                        self.status_message = format!("Loaded {} instances", self.instances.len());
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to load instances: {}", e);
                    }
                }

                // Load available versions for create tab
                if self.available_versions.is_empty() {
                    match rt.block_on(ql_instances::list_versions()) {
                        Ok(versions) => {
                            self.available_versions = versions;
                        }
                        Err(e) => {
                            self.status_message = format!("Failed to load versions: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to create runtime: {}", e);
            }
        }

        self.is_loading = false;
    }

    pub fn set_instance_name(&mut self, name: String) {
        self.new_instance_name = name;
    }

    pub fn toggle_login_mode(&mut self) {
        self.is_login_mode = !self.is_login_mode;
        if !self.is_login_mode {
            self.login_username.clear();
            self.login_password.clear();
        }
    }

    pub fn set_login_username(&mut self, username: String) {
        self.login_username = username;
    }

    pub fn set_login_password(&mut self, password: String) {
        self.login_password = password;
    }

    pub fn get_selected_account(&self) -> Option<&Account> {
        self.accounts.get(self.selected_account)
    }

    pub fn logout_account(&mut self) {
        if let Some(account) = self.accounts.get_mut(self.selected_account) {
            account.is_logged_in = false;
        }
        self.current_account = None;
    }

    pub fn toggle_help_popup(&mut self) {
        self.show_help_popup = !self.show_help_popup;
    }

    fn load_accounts(&mut self) {
        // Load accounts from launcher config
        match LauncherConfig::load_s() {
            Ok(config) => {
                if let Some(config_accounts) = config.accounts {
                    self.accounts = config_accounts.into_iter().map(|(username, config_account)| {
                        let account_type = config_account.account_type
                            .unwrap_or_else(|| "Microsoft".to_string());
                        
                        // For TUI, we'll assume accounts are not logged in initially
                        // since we don't have access to the keyring tokens in this context
                        Account {
                            username,
                            account_type,
                            uuid: config_account.uuid,
                            is_logged_in: false,
                        }
                    }).collect();
                    
                    // Set current account if one was selected
                    if let Some(selected_account) = config.account_selected {
                        self.current_account = Some(selected_account);
                    }
                    
                    // Don't override the instance loading message, just log account loading
                    if self.accounts.is_empty() {
                        self.status_message += " (No accounts configured)";
                    } else {
                        self.status_message = format!("Loaded {} instances and {} accounts", 
                            self.instances.len(), self.accounts.len());
                    }
                } else {
                    self.status_message += " (No accounts configured)";
                }
            }
            Err(err) => {
                self.status_message = format!("Error loading config: {}", err);
            }
        }
    }
}
