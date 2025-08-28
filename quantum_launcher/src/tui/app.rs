// QuantumLauncher TUI - Application State

use std::{error::Error, fmt};
use ql_core::ListEntry;
use crate::config::{LauncherConfig, ConfigAccount};
use tokio::sync::mpsc;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Microsoft,
    ElyBy,
    LittleSkin,
}

impl AccountType {
    pub fn to_string(&self) -> String {
        match self {
            AccountType::Microsoft => "Microsoft".to_string(),
            AccountType::ElyBy => "ElyBy".to_string(),
            AccountType::LittleSkin => "LittleSkin".to_string(),
        }
    }

    pub fn all() -> Vec<AccountType> {
        vec![AccountType::Microsoft, AccountType::ElyBy, AccountType::LittleSkin]
    }
}

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
    pub current_account: Option<String>,
    pub status_message: String,
    pub should_quit: bool,
    pub is_loading: bool,
    // New account creation fields
    pub is_add_account_mode: bool,
    pub new_account_type: AccountType,
    pub new_account_username: String,
    pub new_account_password: String,
    pub selected_account_type: usize,
    // ElyBy specific fields for account creation
    pub new_account_otp: Option<String>,
    pub needs_otp: bool,
    pub show_password: bool,
    pub login_error: Option<String>,
    pub add_account_field_focus: AddAccountFieldFocus, // Track which field is being edited during account creation
    // Auth channel for async authentication
    pub auth_sender: Option<mpsc::UnboundedSender<crate::tui::AuthEvent>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddAccountFieldFocus {
    Username,
    Password,
    Otp,
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
            current_account: None,
            status_message: "Welcome to QuantumLauncher TUI! Press '?' for help, 'q' to quit.".to_string(),
            should_quit: false,
            is_loading: false,
            // Initialize new account creation fields
            is_add_account_mode: false,
            new_account_type: AccountType::Microsoft,
            new_account_username: String::new(),
            new_account_password: String::new(),
            selected_account_type: 0,
            // Initialize ElyBy specific fields for account creation
            new_account_otp: None,
            needs_otp: false,
            show_password: false,
            login_error: None,
            add_account_field_focus: AddAccountFieldFocus::Username,
            auth_sender: None,
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

    /// Switch to next field during account creation (for ElyBy accounts)
    pub fn next_add_account_field(&mut self) {
        if self.new_account_type != AccountType::ElyBy {
            return; // Only ElyBy accounts have field navigation
        }
        
        self.add_account_field_focus = match self.add_account_field_focus {
            AddAccountFieldFocus::Username => AddAccountFieldFocus::Password,
            AddAccountFieldFocus::Password => {
                if self.needs_otp {
                    AddAccountFieldFocus::Otp
                } else {
                    AddAccountFieldFocus::Username
                }
            }
            AddAccountFieldFocus::Otp => AddAccountFieldFocus::Username,
        };
        
        // Update status message to show current field
        let field_name = match self.add_account_field_focus {
            AddAccountFieldFocus::Username => "username/email",
            AddAccountFieldFocus::Password => "password",
            AddAccountFieldFocus::Otp => "OTP code",
        };
        self.status_message = format!("Now editing: {}", field_name);
    }

    /// Add character to current add account field
    pub fn add_char_to_add_account_field(&mut self, c: char) {
        if self.new_account_type != AccountType::ElyBy {
            // For non-ElyBy accounts, just add to username
            self.new_account_username.push(c);
            return;
        }
        
        match self.add_account_field_focus {
            AddAccountFieldFocus::Username => self.new_account_username.push(c),
            AddAccountFieldFocus::Password => self.new_account_password.push(c),
            AddAccountFieldFocus::Otp => {
                if let Some(ref mut otp) = self.new_account_otp {
                    otp.push(c);
                }
            }
        }
    }

    /// Remove character from current add account field
    pub fn remove_char_from_add_account_field(&mut self) {
        if self.new_account_type != AccountType::ElyBy {
            // For non-ElyBy accounts, just remove from username
            self.new_account_username.pop();
            return;
        }
        
        match self.add_account_field_focus {
            AddAccountFieldFocus::Username => { self.new_account_username.pop(); }
            AddAccountFieldFocus::Password => { self.new_account_password.pop(); }
            AddAccountFieldFocus::Otp => {
                if let Some(ref mut otp) = self.new_account_otp {
                    otp.pop();
                }
            }
        }
    }

    /// Toggle password visibility
    pub fn toggle_password_visibility(&mut self) {
        self.show_password = !self.show_password;
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

    pub fn toggle_add_account_mode(&mut self) {
        self.is_add_account_mode = !self.is_add_account_mode;
        if !self.is_add_account_mode {
            // Reset all fields when exiting add account mode
            self.new_account_username.clear();
            self.new_account_password.clear();
            self.selected_account_type = 0;
            self.new_account_type = AccountType::Microsoft;
            // Reset ElyBy specific fields
            self.new_account_otp = None;
            self.needs_otp = false;
            self.login_error = None;
            self.show_password = false;
            self.add_account_field_focus = AddAccountFieldFocus::Username;
        }
    }

    pub fn next_account_type(&mut self) {
        let account_types = AccountType::all();
        if !account_types.is_empty() {
            self.selected_account_type = (self.selected_account_type + 1) % account_types.len();
            self.new_account_type = account_types[self.selected_account_type].clone();
        }
    }

    pub fn prev_account_type(&mut self) {
        let account_types = AccountType::all();
        if !account_types.is_empty() && self.selected_account_type > 0 {
            self.selected_account_type -= 1;
        } else if !account_types.is_empty() {
            self.selected_account_type = account_types.len() - 1;
        }
        self.new_account_type = account_types[self.selected_account_type].clone();
    }

    pub fn add_new_account(&mut self) {
        if self.new_account_username.is_empty() {
            self.status_message = "Please enter a username".to_string();
            return;
        }

        match self.new_account_type {
            AccountType::Microsoft => {
                // For Microsoft accounts, just add with OAuth2 placeholder
                let new_account = Account {
                    username: self.new_account_username.clone(),
                    account_type: self.new_account_type.to_string(),
                    uuid: "00000000-0000-0000-0000-000000000000".to_string(), // Placeholder UUID
                    is_logged_in: false,
                };
                
                self.accounts.push(new_account);
                self.status_message = format!("Added new {} account: {}", 
                    self.new_account_type.to_string(), self.new_account_username);
                
                // Reset and exit add account mode
                self.toggle_add_account_mode();
            }
            AccountType::ElyBy => {
                // For ElyBy accounts, validate password and potentially handle OTP
                if self.new_account_password.is_empty() {
                    self.login_error = Some("Password is required for ElyBy accounts".to_string());
                    return;
                }
                
                // Start real authentication process
                self.start_elyby_authentication();
            }
            AccountType::LittleSkin => {
                // Similar to ElyBy but for LittleSkin
                if self.new_account_password.is_empty() {
                    self.login_error = Some("Password is required for LittleSkin accounts".to_string());
                    return;
                }
                
                // For now, just add like Microsoft until we implement LittleSkin auth
                let new_account = Account {
                    username: self.new_account_username.clone(),
                    account_type: self.new_account_type.to_string(),
                    uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                    is_logged_in: false,
                };
                
                self.accounts.push(new_account);
                self.status_message = format!("Added new {} account: {}", 
                    self.new_account_type.to_string(), self.new_account_username);
                
                self.toggle_add_account_mode();
            }
        }
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

    /// Set the authentication channel for async operations
    pub fn set_auth_channel(&mut self, sender: mpsc::UnboundedSender<crate::tui::AuthEvent>) {
        self.auth_sender = Some(sender);
    }

    /// Handle authentication events from async operations
    pub fn handle_auth_event(&mut self, event: crate::tui::AuthEvent) {
        match event {
            crate::tui::AuthEvent::LoginStarted => {
                self.status_message = "🔐 Authenticating with ElyBy...".to_string();
                self.is_loading = true;
            }
            crate::tui::AuthEvent::LoginSuccess { account_data } => {
                self.is_loading = false;
                
                // Save account to config and keyring (like iced implementation)
                if let Err(err) = self.save_authenticated_account(&account_data) {
                    self.login_error = Some(format!("Failed to save account: {}", err));
                    self.status_message = format!("❌ Authentication succeeded but failed to save: {}", err);
                    return;
                }
                
                // Create account for TUI display
                let account = Account {
                    username: account_data.nice_username.clone(),
                    account_type: account_data.account_type.to_string(),
                    uuid: account_data.uuid.clone(),
                    is_logged_in: true,
                };
                
                self.accounts.push(account);
                
                // Set as current/default account (like iced implementation)
                self.current_account = Some(account_data.get_username_modified());
                
                self.status_message = format!("✅ Successfully authenticated and saved as {} (set as default)", account_data.nice_username);
                
                // Reset and exit add account mode
                self.toggle_add_account_mode();
            }
            crate::tui::AuthEvent::LoginNeedsOtp => {
                self.is_loading = false;
                self.needs_otp = true;
                self.new_account_otp = Some(String::new());
                self.add_account_field_focus = AddAccountFieldFocus::Otp;
                self.status_message = "📱 Two-factor authentication required. Enter your OTP code.".to_string();
            }
            crate::tui::AuthEvent::LoginError { error } => {
                self.is_loading = false;
                self.login_error = Some(error.clone());
                self.status_message = format!("❌ Authentication failed: {}", error);
            }
        }
    }

    /// Start ElyBy authentication process
    pub fn start_elyby_authentication(&mut self) {
        if let Some(sender) = &self.auth_sender {
            // Clear any previous errors
            self.login_error = None;
            
            // Prepare password with OTP if needed
            let mut password = self.new_account_password.clone();
            if let Some(ref otp) = self.new_account_otp {
                if !otp.is_empty() {
                    password.push(':');
                    password.push_str(otp);
                }
            }

            let username = self.new_account_username.clone();
            let sender_clone = sender.clone();
            
            // Send login started event
            let _ = sender.send(crate::tui::AuthEvent::LoginStarted);
            
            // Spawn authentication task
            tokio::spawn(async move {
                match ql_instances::auth::yggdrasil::login_new(
                    username,
                    password,
                    ql_instances::auth::AccountType::ElyBy,
                ).await {
                    Ok(ql_instances::auth::yggdrasil::Account::Account(account_data)) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::LoginSuccess { account_data });
                    }
                    Ok(ql_instances::auth::yggdrasil::Account::NeedsOTP) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::LoginNeedsOtp);
                    }
                    Err(err) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::LoginError {
                            error: err.to_string(),
                        });
                    }
                }
            });
        }
    }

    /// Set the currently selected account as the default account
    pub fn set_default_account(&mut self) {
        if self.accounts.is_empty() {
            self.status_message = "No accounts available to set as default.".to_string();
            return;
        }

        let selected_account = &self.accounts[self.selected_account];
        
        // Create the username with type modifier (like iced does)
        let username_modified = if selected_account.account_type == "ElyBy" {
            format!("{} (elyby)", selected_account.username)
        } else if selected_account.account_type == "LittleSkin" {
            format!("{} (littleskin)", selected_account.username)
        } else {
            selected_account.username.clone()
        };
        
        // Update current account
        self.current_account = Some(username_modified.clone());
        
        // Save to config
        if let Err(err) = self.save_default_account_to_config(&username_modified) {
            self.status_message = format!("❌ Failed to save default account: {}", err);
        } else {
            self.status_message = format!("✅ Set {} as default account", selected_account.username);
        }
    }

    /// Save the default account selection to config
    fn save_default_account_to_config(&self, username_modified: &str) -> Result<(), String> {
        let mut config = LauncherConfig::load_s().unwrap_or_default();
        config.account_selected = Some(username_modified.to_string());
        self.save_config_sync(&config).map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Save authenticated account to config and keyring (like iced implementation)
    fn save_authenticated_account(&mut self, account_data: &ql_instances::auth::AccountData) -> Result<(), String> {
        // Load current config
        let mut config = LauncherConfig::load_s().unwrap_or_default();
        
        // Get or create accounts section
        let accounts = config.accounts.get_or_insert_with(Default::default);
        
        // Create username with type modifier (like iced does)
        let username_modified = account_data.get_username_modified();
        
        // Add account to config
        accounts.insert(
            username_modified.clone(),
            ConfigAccount {
                uuid: account_data.uuid.clone(),
                skin: None,
                account_type: Some(account_data.account_type.to_string()),
                keyring_identifier: Some(account_data.username.clone()),
                username_nice: Some(account_data.nice_username.clone()),
            },
        );
        
        // Set as selected account
        config.account_selected = Some(username_modified);
        
        // Save config
        self.save_config_sync(&config).map_err(|e| format!("Failed to save config: {}", e))?;
        
        // Note: The keyring token is already saved by the yggdrasil::login_new function
        // in ql_instances/src/auth/yggdrasil/mod.rs line 59:
        // entry.set_password(&account_response.accessToken)?;
        
        Ok(())
    }

    /// Save config synchronously (helper for TUI)
    fn save_config_sync(&self, config: &LauncherConfig) -> Result<(), ql_core::JsonFileError> {
        let config_path = ql_core::file_utils::get_launcher_dir()?.join("config.json");
        let config_json = serde_json::to_string(config)
            .map_err(|error| ql_core::JsonFileError::SerdeError(ql_core::JsonError::To { error }))?;
        std::fs::write(&config_path, config_json.as_bytes())
            .map_err(|error| ql_core::JsonFileError::Io(ql_core::IoError::Io { 
                error: error.to_string(), 
                path: config_path.clone() 
            }))?;
        Ok(())
    }
}
