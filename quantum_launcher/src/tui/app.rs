// QuantumLauncher TUI - Application State

use std::{error::Error, fmt, sync::{Arc, Mutex}, collections::HashSet};
use ql_core::ListEntry;
use crate::config::{LauncherConfig, ConfigAccount};
use tokio::sync::mpsc;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Microsoft,
    ElyBy,
    LittleSkin,
    Offline,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccountType::Microsoft => "Microsoft (Coming Soon)",
            AccountType::ElyBy => "ElyBy",
            AccountType::LittleSkin => "LittleSkin (Coming Soon)",
            AccountType::Offline => "Offline",
        };
        write!(f, "{}", name)
    }
}

impl AccountType {
    pub fn all() -> Vec<AccountType> {
        vec![
            AccountType::Microsoft,
            AccountType::ElyBy, 
            AccountType::LittleSkin,
            AccountType::Offline
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabId {
    Instances,
    Create,
    Settings,
    Accounts,
    Logs,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub account_type: String,
    #[allow(dead_code)] // May be used in future for account management
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
            TabId::Logs => write!(f, "Logs"),
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub name: String,
    pub version: String,
    pub loader: String,
    pub is_running: bool,
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
    // Game logs storage
    pub game_logs: Vec<String>,
    // Terminal refresh tracking
    pub needs_forced_refresh: bool,
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
            new_account_type: AccountType::ElyBy, // Default to ElyBy as the primary working option
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
            game_logs: Vec::new(),
            needs_forced_refresh: false,
        };
        
        // Load instances and accounts on startup
        app.refresh();
        app.load_accounts();
        app
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    #[allow(dead_code)] // May be used for graceful shutdown
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabId::Instances => TabId::Create,
            TabId::Create => TabId::Settings,
            TabId::Settings => TabId::Accounts,
            TabId::Accounts => TabId::Logs,
            TabId::Logs => TabId::Instances,
        };
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabId::Instances => TabId::Logs,
            TabId::Create => TabId::Instances,
            TabId::Settings => TabId::Create,
            TabId::Accounts => TabId::Settings,
            TabId::Logs => TabId::Accounts,
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
                    // Check if we have a default account or if we need one
                    if self.current_account.is_none() && !self.can_launch_offline() {
                        self.status_message = "⚠️ No default account set. Add an account (press 'a' then 'n') or set one as default (press 'd')".to_string();
                        return;
                    }
                    
                    // Start launching the instance
                    let instance_name = instance.name.clone();
                    self.launch_instance(&instance_name);
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

    /// Launch the currently selected instance (for Shift+Enter)
    pub fn launch_selected_instance(&mut self) {
        if let Some(instance) = self.instances.get(self.selected_instance) {
            // Start launching the instance (bypass account checks for quick launch)
            let instance_name = instance.name.clone();
            self.launch_instance(&instance_name);
        }
    }

    pub fn refresh(&mut self) {
        use std::path::PathBuf;
        use crate::state::get_entries;
        use ql_core::json::{InstanceConfigJson, VersionDetails};
        use ql_core::file_utils;

        self.is_loading = true;
        self.status_message = "Refreshing...".to_string();

        // Preserve running status
        let running_instances: HashSet<String> = self.instances
            .iter()
            .filter(|i| i.is_running)
            .map(|i| i.name.clone())
            .collect();

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
                                is_running: running_instances.contains(&name),
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

    #[allow(dead_code)] // May be used for automated instance creation
    pub fn set_instance_name(&mut self, name: String) {
        self.new_instance_name = name;
    }

    /// Switch to next field during account creation (for ElyBy and LittleSkin accounts)
    pub fn next_add_account_field(&mut self) {
        if self.new_account_type != AccountType::ElyBy && self.new_account_type != AccountType::LittleSkin {
            return; // Only ElyBy and LittleSkin accounts have field navigation
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
        if self.new_account_type != AccountType::ElyBy && self.new_account_type != AccountType::LittleSkin {
            // For non-ElyBy/LittleSkin accounts, just add to username
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
        if self.new_account_type != AccountType::ElyBy && self.new_account_type != AccountType::LittleSkin {
            // For non-ElyBy/LittleSkin accounts, just remove from username
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
        if self.selected_account >= self.accounts.len() {
            self.status_message = "No account selected to logout".to_string();
            return;
        }

        let account = &self.accounts[self.selected_account];
        
        // Don't try to logout offline accounts - they can't be logged out
        if account.account_type == "Offline" {
            self.status_message = "Cannot logout offline accounts - they are always ready to use".to_string();
            return;
        }

        let username = account.username.clone();
        let account_type_str = account.account_type.clone();

        // Convert account type string to auth AccountType enum
        let account_type = match account_type_str.as_str() {
            "ElyBy" => ql_instances::auth::AccountType::ElyBy,
            "LittleSkin" => ql_instances::auth::AccountType::LittleSkin,
            _ => ql_instances::auth::AccountType::Microsoft,
        };

        // Get the keyring username (without the type modifier)
        let keyring_username = match account_type {
            ql_instances::auth::AccountType::ElyBy => {
                username.strip_suffix(" (elyby)").unwrap_or(&username).to_string()
            }
            ql_instances::auth::AccountType::LittleSkin => {
                username.strip_suffix(" (littleskin)").unwrap_or(&username).to_string()
            }
            ql_instances::auth::AccountType::Microsoft => username.clone(),
        };

        // Call the proper logout function to remove keyring entry
        if let Err(err) = ql_instances::auth::logout(&keyring_username, account_type) {
            self.status_message = format!("Failed to logout keyring for {}: {}", keyring_username, err);
        } else {
            self.status_message = format!("Successfully logged out: {}", account.username);
        }

        // Remove from config file
        if let Err(err) = self.remove_account_from_config(&username) {
            self.status_message = format!("Warning: Failed to remove account from config: {}", err);
        }

        // Remove from in-memory accounts list
        self.accounts.remove(self.selected_account);

        // If this was the current/default account, clear it
        if let Some(ref current) = self.current_account {
            if current == &username {
                self.current_account = None;
            }
        }

        // Adjust selected account index if needed
        if self.selected_account >= self.accounts.len() && !self.accounts.is_empty() {
            self.selected_account = self.accounts.len() - 1;
        }
        if self.accounts.is_empty() {
            self.selected_account = 0;
        }
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
            self.new_account_type = AccountType::ElyBy; // Default to ElyBy as the primary working option
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
            if self.selected_account_type < account_types.len() {
                self.new_account_type = account_types[self.selected_account_type].clone();
            }
        }
    }

    pub fn prev_account_type(&mut self) {
        let account_types = AccountType::all();
        if !account_types.is_empty() && self.selected_account_type > 0 {
            self.selected_account_type -= 1;
        } else if !account_types.is_empty() {
            self.selected_account_type = account_types.len() - 1;
        }
        if self.selected_account_type < account_types.len() {
            self.new_account_type = account_types[self.selected_account_type].clone();
        }
    }

    pub fn add_new_account(&mut self) {
        if self.new_account_username.is_empty() {
            self.status_message = "Please enter a username".to_string();
            return;
        }

        match self.new_account_type {
            AccountType::Microsoft => {
                self.status_message = "Microsoft accounts are not implemented yet - coming soon!".to_string();
                self.login_error = Some("Microsoft authentication not yet available".to_string());
                return;
            }
            AccountType::LittleSkin => {
                self.status_message = "LittleSkin accounts are not implemented yet - coming soon!".to_string();
                self.login_error = Some("LittleSkin authentication not yet available".to_string());
                return;
            }
            AccountType::Offline => {
                // For offline accounts, just add with the specified username
                let new_account = Account {
                    username: self.new_account_username.clone(),
                    account_type: self.new_account_type.to_string(),
                    uuid: "00000000-0000-0000-0000-000000000000".to_string(), // Placeholder UUID for offline
                    is_logged_in: true, // Offline accounts are always "logged in"
                };
                
                // Save to config file to persist between launches
                if let Err(err) = self.save_offline_account(&new_account) {
                    self.status_message = format!("Warning: Failed to save offline account to config: {}", err);
                } else {
                    self.status_message = format!("Added offline account: {} (ready for launching)", 
                        self.new_account_username);
                }
                
                self.accounts.push(new_account);
                
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
        }
    }

    fn load_accounts(&mut self) {
        // Load accounts from launcher config (iced UI logic)
        match LauncherConfig::load_s() {
            Ok(config) => {
                if let Some(config_accounts) = config.accounts {
                    self.accounts = config_accounts.iter().map(|(username_key, config_account)| {
                        let account_type = config_account.account_type.clone().unwrap_or_else(|| "Microsoft".to_string());
                        
                        // Determine if account is logged in
                        let is_logged_in = if account_type == "Offline" {
                            true // Offline accounts are always logged in
                        } else {
                            // For ElyBy/LittleSkin accounts, check if they have a valid keyring token
                            let keyring_username = if let Some(keyring_id) = &config_account.keyring_identifier {
                                keyring_id.clone()
                            } else {
                                // Fallback to old behavior for backwards compatibility
                                match account_type.as_str() {
                                    "ElyBy" => username_key.strip_suffix(" (elyby)").unwrap_or(username_key).to_string(),
                                    "LittleSkin" => username_key.strip_suffix(" (littleskin)").unwrap_or(username_key).to_string(),
                                    _ => username_key.clone(),
                                }
                            };
                            
                            let auth_account_type = match account_type.as_str() {
                                "ElyBy" => ql_instances::auth::AccountType::ElyBy,
                                "LittleSkin" => ql_instances::auth::AccountType::LittleSkin,
                                _ => ql_instances::auth::AccountType::Microsoft,
                            };
                            
                            // Check if token exists in keyring
                            ql_instances::auth::read_refresh_token(&keyring_username, auth_account_type).is_ok()
                        };
                        
                        Account {
                            username: username_key.clone(), // Store full key (with suffix)
                            account_type,
                            uuid: config_account.uuid.clone(),
                            is_logged_in,
                        }
                    }).collect();
                    // Set current account if one was selected
                    if let Some(selected_account) = config.account_selected {
                        self.current_account = Some(selected_account);
                    }
                    if self.accounts.is_empty() {
                        self.status_message += " (No accounts configured)";
                    } else {
                        self.status_message = format!("Loaded {} instances and {} accounts", self.instances.len(), self.accounts.len());
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
            crate::tui::AuthEvent::LaunchStarted(instance_name) => {
                self.status_message = format!("🚀 Launching {}...", instance_name);
                self.is_loading = true;
                self.needs_forced_refresh = true; // Force refresh to overwrite debug output
                self.add_log(format!("[{}] Launch started for instance: {}", 
                    chrono::Local::now().format("%H:%M:%S"), instance_name));
            }
            crate::tui::AuthEvent::LaunchSuccess(instance_name, child) => {
                self.is_loading = false;
                self.needs_forced_refresh = true; // Force refresh to overwrite debug output
                let pid = child.lock().map(|c| c.id()).unwrap_or(None);
                let msg = match pid {
                    Some(pid) => format!("✅ Successfully launched {}! Game is running with PID: {}", instance_name, pid),
                    None => format!("✅ Successfully launched {}! Game is running in background.", instance_name),
                };
                self.status_message = msg.clone();
                self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
                
                // Mark the instance as running
                if let Some(instance) = self.instances.iter_mut().find(|i| i.name == instance_name) {
                    instance.is_running = true;
                }
                
                // Spawn a task to monitor the process
                if let Some(sender) = &self.auth_sender {
                    let sender_clone = sender.clone();
                    let instance_name_clone = instance_name.clone();
                    let child_clone = child.clone();
                    tokio::spawn(async move {
                        // Poll the process status without holding the lock
                        loop {
                            {
                                let mut child_guard = child_clone.lock().unwrap();
                                if let Ok(Some(_)) = child_guard.try_wait() {
                                    break;
                                }
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                        let _ = sender_clone.send(crate::tui::AuthEvent::LaunchEnded(instance_name_clone));
                    });
                }
            }
            crate::tui::AuthEvent::LaunchError(instance_name, error) => {
                self.is_loading = false;
                self.needs_forced_refresh = true; // Force refresh to overwrite debug output
                let msg = format!("❌ Failed to launch {}: {}", instance_name, error);
                self.status_message = msg.clone();
                self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
            }
            crate::tui::AuthEvent::LaunchEnded(instance_name) => {
                // Mark the instance as stopped
                if let Some(instance) = self.instances.iter_mut().find(|i| i.name == instance_name) {
                    instance.is_running = false;
                }
                let msg = format!("🛑 {} has stopped", instance_name);
                self.status_message = msg.clone();
                self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
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
        
        // Use the account's username directly since it's already stored with the proper format
        // from when the accounts were loaded from the config
        let default_key = selected_account.username.clone();
        
        // Update current account
        self.current_account = Some(default_key.clone());
        
        // Save to config
        if let Err(err) = self.save_default_account_to_config(&default_key) {
            self.status_message = format!("❌ Failed to save default account: {}", err);
        } else {
            self.status_message = format!("✅ Set {} as default account", selected_account.username);
        }
    }

    /// Save the default account selection to config
    fn save_default_account_to_config(&self, default_key: &str) -> Result<(), String> {
        let mut config = LauncherConfig::load_s().unwrap_or_default();
        config.account_selected = Some(default_key.to_string());
        self.save_config_sync(&config)
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
        self.save_config_sync(&config)?;
        
        // Note: The keyring token is already saved by the yggdrasil::login_new function
        // in ql_instances/src/auth/yggdrasil/mod.rs line 59:
        // entry.set_password(&account_response.accessToken)?;
        
        Ok(())
    }

    /// Save config synchronously (helper for TUI)
    fn save_config_sync(&self, config: &LauncherConfig) -> Result<(), String> {
        let config_path = match ql_core::file_utils::get_launcher_dir() {
            Ok(dir) => dir.join("config.json"),
            Err(e) => return Err(format!("Failed to get launcher directory: {}", e)),
        };
        
        let config_str = match serde_json::to_string_pretty(config) {
            Ok(str) => str,
            Err(e) => return Err(format!("Failed to serialize config: {}", e)),
        };
        
        match std::fs::write(&config_path, config_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write config: {}", e)),
        }
    }
    
    /// Remove account from config file
    fn remove_account_from_config(&self, username: &str) -> Result<(), String> {
        let mut config = LauncherConfig::load_s().unwrap_or_default();
        
        // Remove from accounts section
        if let Some(accounts) = &mut config.accounts {
            accounts.remove(username);
        }
        
        // If this was the selected account, clear the selection
        if config.account_selected.as_deref() == Some(username) {
            config.account_selected = None;
        }
        
        // Save config
        self.save_config_sync(&config)
    }
    
    /// Save offline account to config (similar to save_authenticated_account but for offline accounts)
    fn save_offline_account(&self, account: &Account) -> Result<(), String> {
        // Load current config
        let mut config = LauncherConfig::load_s().unwrap_or_default();
        
        // Get or create accounts section
        let accounts = config.accounts.get_or_insert_with(Default::default);
        
        // For offline accounts, use the username directly as the key
        let username = account.username.clone();
        
        // Add account to config
        accounts.insert(
            username.clone(),
            ConfigAccount {
                uuid: account.uuid.clone(),
                skin: None,
                account_type: Some("Offline".to_string()),
                keyring_identifier: None, // Offline accounts don't need keyring
                username_nice: Some(username.clone()),
            },
        );
        
        // Set as selected account if requested
        if self.accounts.is_empty() {
            // If this is the first account, set it as selected
            config.account_selected = Some(username);
        }
        
        // Save config
        self.save_config_sync(&config).map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Check if we can launch offline (with offline accounts or Microsoft accounts)
    pub fn can_launch_offline(&self) -> bool {
        // Allow offline launch if we have any Offline account
        // Microsoft option is commented out
        self.accounts.iter().any(|acc| acc.account_type == "Offline")
    }

    /// Launch a Minecraft instance with output capture
    fn launch_instance(&mut self, instance_name: &str) {
        self.status_message = format!("🚀 Launching instance: {}", instance_name);
        
        // Get account data if we have a current account or selected account
        let account_data = self.get_account_data_for_launch();
        let (username, _display_name) = if let Some(acc) = &account_data {
            // For authenticated accounts, use the username from account data (should be clean)
            let clean_username = self.get_clean_username_for_launch(&acc.username, &acc.nice_username, &acc.account_type);
            self.status_message = format!("🚀 Launching with authenticated account: {} ({})", 
                acc.nice_username, acc.account_type);
            (clean_username, acc.nice_username.clone())
        } else {
            // Try to use the currently selected account even if not set as default
            if !self.accounts.is_empty() && self.selected_account < self.accounts.len() {
                let selected_account = &self.accounts[self.selected_account];
                
                // Special handling for offline accounts
                if selected_account.account_type == "Offline" {
                    let clean_username = self.get_clean_username_for_selected_account(selected_account);
                    self.status_message = format!("🚀 Launching with offline account: {}", clean_username);
                    (clean_username, selected_account.username.clone())
                } else {
                    // Try to get account data for this selected account again, but with fallback
                    let fallback_account_data = self.get_selected_account_data_fallback();
                    if let Some(acc) = fallback_account_data {
                        let clean_username = self.get_clean_username_for_launch(&acc.username, &acc.nice_username, &acc.account_type);
                        self.status_message = format!("🚀 Launching with selected account: {} ({})", 
                            acc.nice_username, selected_account.account_type);
                        (clean_username, acc.nice_username.clone())
                    } else {
                        // No auth data available, generate clean username from account info
                        let clean_username = self.get_clean_username_for_selected_account(&selected_account);
                        self.status_message = format!("⚠️ Using selected account: {} ({}) in offline mode. Account may need re-authentication.", 
                            clean_username, selected_account.account_type);
                        (clean_username, selected_account.username.clone())
                    }
                }
            } else {
                // No accounts available, launch in offline mode
                self.status_message = "⚠️ No accounts available. Launching in offline mode as 'Player'. Add an account with 'a' then 'n'.".to_string();
                ("Player".to_string(), "Player".to_string())
            }
        };
        
        // Spawn launch task similar to authentication
        if let Some(sender) = &self.auth_sender {
            let sender_clone = sender.clone();
            let instance_name = instance_name.to_string();
            // Use the final account_data that we determined above (could be from default, selected, or None)
            let final_account_data = if account_data.is_some() {
                account_data
            } else if !self.accounts.is_empty() && self.selected_account < self.accounts.len() {
                let selected_account = &self.accounts[self.selected_account];
                
                // If this is an offline account, we don't need account data
                if selected_account.account_type == "Offline" {
                    None
                } else {
                    // Try to get account data for selected account as last resort
                    let fallback_data = self.get_selected_account_data_fallback();
                    if fallback_data.is_some() {
                        fallback_data
                    } else {
                        // Create minimal account data for authlib-injector if this is ElyBy/LittleSkin
                        self.create_minimal_account_data_for_authlib(selected_account)
                    }
                }
            } else {
                None
            };
            
            // Send launch started event
            let _ = sender.send(crate::tui::AuthEvent::LaunchStarted(instance_name.clone()));
            
            // Spawn launch task
            tokio::spawn(async move {
                // Create a custom launch that suppresses debug output
                let result = Self::launch_with_suppressed_output(
                    instance_name.clone(),
                    username,
                    final_account_data,
                    sender_clone.clone(),
                ).await;
                
                match result {
                    Ok(child) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::LaunchSuccess(instance_name, child));
                    }
                    Err(e) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::LaunchError(instance_name, e.to_string()));
                    }
                }
            });
        } else {
            self.status_message = "❌ Authentication system not initialized".to_string();
        }
    }

    /// Launch function that suppresses debug output to prevent TUI spam
    async fn launch_with_suppressed_output(
        instance_name: String,
        username: String,
        account_data: Option<ql_instances::auth::AccountData>,
        sender: mpsc::UnboundedSender<crate::tui::AuthEvent>,
    ) -> Result<Arc<Mutex<tokio::process::Child>>, Box<dyn Error + Send + Sync>> {
        // Add log entry that we're starting
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let launch_mode = if let Some(ref acc) = account_data {
            // We have account data - check if we have a valid access token
            if acc.access_token.is_some() {
                format!("with {} account: {}", acc.account_type, acc.nice_username)
            } else {
                format!("in offline mode as: {}", acc.nice_username)
            }
        } else {
            format!("in offline mode as: {}", username)
        };
        let _ = sender.send(crate::tui::AuthEvent::LaunchStarted(
            format!("[{}] Preparing to launch {} {}", timestamp, instance_name, launch_mode)
        ));
        
        // Set environment variable to suppress ql_instances debug output (if it supports this)
        std::env::set_var("QL_QUIET_LAUNCH", "true");
        
        match ql_instances::launch(
            instance_name.clone(),
            username.clone(),
            None, // No progress sender for TUI
            account_data,
            None, // No global settings
            Vec::new(), // No extra java args
        ).await {
            Ok(child_arc) => {
                // Brief delay to allow any debug output to finish before we refresh the TUI
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // Get PID for logging
                let pid = child_arc.lock().map(|child| child.id()).unwrap_or(None);
                let timestamp = chrono::Local::now().format("%H:%M:%S");
                let _ = sender.send(crate::tui::AuthEvent::LaunchStarted(
                    format!("[{}] Minecraft process started with PID: {:?}", timestamp, pid)
                ));
                
                // Add success message to logs
                let _ = sender.send(crate::tui::AuthEvent::LaunchStarted(
                    format!("[{}] Launch completed successfully!", timestamp)
                ));
                
                Ok(child_arc)
            }
            Err(e) => {
                let timestamp = chrono::Local::now().format("%H:%M:%S");
                let _ = sender.send(crate::tui::AuthEvent::LaunchStarted(
                    format!("[{}] Launch failed: {}", timestamp, e)
                ));
                Err(Box::new(e) as Box<dyn Error + Send + Sync>)
            }
        }
    }

    /// Get account data for launching (simplified version for TUI)
    // Use iced UI logic for account data retrieval
    fn get_account_data_for_launch(&self) -> Option<ql_instances::auth::AccountData> {
        // Use iced UI logic: get_selected_account_data
        // Direct copy of iced UI logic for account selection
        if self.accounts.is_empty() || self.selected_account >= self.accounts.len() {
            return None;
        }
        let account = &self.accounts[self.selected_account];
        
        // For offline accounts, no account data needed - will use username directly
        if account.account_type == "Offline" {
            // Return None for Offline accounts - the launch system will handle this properly
            // by using the username directly for offline mode
            return None;
        }
        
        if let Ok(config) = LauncherConfig::load_s() {
            if let Some(config_accounts) = config.accounts {
                if let Some(config_account) = config_accounts.get(&account.username) {
                    let account_type = match account.account_type.as_str() {
                        "ElyBy" => ql_instances::auth::AccountType::ElyBy,
                        "LittleSkin" => ql_instances::auth::AccountType::LittleSkin,
                        _ => ql_instances::auth::AccountType::Microsoft,
                    };
                    let keyring_username = if let Some(keyring_id) = &config_account.keyring_identifier {
                        keyring_id.clone()
                    } else {
                        match account_type {
                            ql_instances::auth::AccountType::ElyBy => account.username.strip_suffix(" (elyby)").unwrap_or(&account.username).to_string(),
                            ql_instances::auth::AccountType::LittleSkin => account.username.strip_suffix(" (littleskin)").unwrap_or(&account.username).to_string(),
                            ql_instances::auth::AccountType::Microsoft => account.username.clone(),
                        }
                    };
                    match ql_instances::auth::read_refresh_token(&keyring_username, account_type) {
                        Ok(refresh_token) => {
                            return Some(ql_instances::auth::AccountData {
                                access_token: Some("0".to_string()), // Fixed token
                                uuid: config_account.uuid.clone(),
                                refresh_token,
                                needs_refresh: true,
                                account_type,
                                username: keyring_username.clone(),
                                nice_username: config_account.username_nice.clone().unwrap_or(keyring_username),
                            });
                        }
                        Err(_) => {
                            // If token not found, return minimal AccountData for ElyBy/LittleSkin
                            if matches!(account_type, ql_instances::auth::AccountType::ElyBy | ql_instances::auth::AccountType::LittleSkin) {
                                let nice_username = config_account.username_nice.clone().unwrap_or_else(|| account.username.clone());
                                return Some(ql_instances::auth::AccountData {
                                    access_token: Some("0".to_string()), // Fixed token
                                    uuid: config_account.uuid.clone(),
                                    refresh_token: String::new(),
                                    needs_refresh: true,
                                    account_type,
                                    username: account.username.clone(),
                                    nice_username,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get account data for the currently selected account (fallback when no default is set)
    // Use iced UI logic for fallback account data
    fn get_selected_account_data_fallback(&self) -> Option<ql_instances::auth::AccountData> {
        self.get_selected_account_data()
    }
    // Iced UI logic for account data retrieval
    fn get_selected_account_data(&self) -> Option<ql_instances::auth::AccountData> {
        if self.accounts.is_empty() || self.selected_account >= self.accounts.len() {
            return None;
        }
        let account = &self.accounts[self.selected_account];
        
        // For offline accounts, create minimal account data
        if account.account_type == "Offline" {
            return Some(ql_instances::auth::AccountData {
                access_token: Some("0".to_string()), // Use a dummy token instead of None
                uuid: account.uuid.clone(),
                refresh_token: String::new(),
                needs_refresh: false, // Offline accounts don't need refresh
                account_type: ql_instances::auth::AccountType::Microsoft, // Use Microsoft but with dummy token
                username: account.username.clone(),
                nice_username: account.username.clone(),
            });
        }
        
        if let Ok(config) = LauncherConfig::load_s() {
            if let Some(config_accounts) = config.accounts {
                if let Some(config_account) = config_accounts.get(&account.username) {
                    let account_type = match account.account_type.as_str() {
                        "ElyBy" => ql_instances::auth::AccountType::ElyBy,
                        "LittleSkin" => ql_instances::auth::AccountType::LittleSkin,
                        _ => ql_instances::auth::AccountType::Microsoft,
                    };
                    let keyring_username = if let Some(keyring_id) = &config_account.keyring_identifier {
                        keyring_id.clone()
                    } else {
                        match account_type {
                            ql_instances::auth::AccountType::ElyBy => account.username.strip_suffix(" (elyby)").unwrap_or(&account.username).to_string(),
                            ql_instances::auth::AccountType::LittleSkin => account.username.strip_suffix(" (littleskin)").unwrap_or(&account.username).to_string(),
                            ql_instances::auth::AccountType::Microsoft => account.username.clone(),
                        }
                    };
                    match ql_instances::auth::read_refresh_token(&keyring_username, account_type) {
                        Ok(refresh_token) => {
                            return Some(ql_instances::auth::AccountData {
                                access_token: Some("0".to_string()), // Fixed token
                                uuid: config_account.uuid.clone(),
                                refresh_token,
                                needs_refresh: true,
                                account_type,
                                username: keyring_username.clone(),
                                nice_username: config_account.username_nice.clone().unwrap_or(keyring_username),
                            });
                        }
                        Err(_) => {
                            // If token not found, return minimal AccountData for ElyBy/LittleSkin
                            if matches!(account_type, ql_instances::auth::AccountType::ElyBy | ql_instances::auth::AccountType::LittleSkin) {
                                let nice_username = config_account.username_nice.clone().unwrap_or_else(|| account.username.clone());
                                return Some(ql_instances::auth::AccountData {
                                    access_token: Some("0".to_string()), // Fixed token
                                    uuid: config_account.uuid.clone(),
                                    refresh_token: String::new(),
                                    needs_refresh: true,
                                    account_type,
                                    username: account.username.clone(),
                                    nice_username,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Create minimal AccountData for authlib-injector when no authentication is available
    fn create_minimal_account_data_for_authlib(&self, account: &Account) -> Option<ql_instances::auth::AccountData> {
        let account_type = match account.account_type.as_str() {
            "ElyBy" => ql_instances::auth::AccountType::ElyBy,
            "LittleSkin" => ql_instances::auth::AccountType::LittleSkin,
            _ => return None, // Only create for ElyBy/LittleSkin
        };

        // Get nice username from config if available
        let nice_username = if let Ok(config) = LauncherConfig::load_s() {
            if let Some(config_accounts) = config.accounts {
                if let Some(config_account) = config_accounts.get(&account.username) {
                    config_account.username_nice.clone().unwrap_or_else(|| account.username.clone())
                } else {
                    account.username.clone()
                }
            } else {
                account.username.clone()
            }
        } else {
            account.username.clone()
        };

        Some(ql_instances::auth::AccountData {
            access_token: Some("0".to_string()), // Fixed token // No access token for offline mode
            uuid: account.uuid.clone(),
            refresh_token: String::new(), // Empty refresh token
            needs_refresh: true, // Mark as needing refresh
            username: account.username.clone(),
            nice_username,
            account_type,
        })
    }

    /// Generate a clean username for launching based on account type and available data
    fn get_clean_username_for_launch(&self, username: &str, nice_username: &str, account_type: &ql_instances::auth::AccountType) -> String {
        match account_type {
            ql_instances::auth::AccountType::ElyBy | ql_instances::auth::AccountType::LittleSkin => {
                // For ElyBy/LittleSkin, use the nice_username if it doesn't contain spaces or special chars
                // Otherwise, try to extract a clean part from the username (email)
                if nice_username.chars().all(|c| c.is_alphanumeric() || c == '_') && nice_username.len() <= 16 {
                    nice_username.to_string()
                } else {
                    // Extract username part from email or use first part
                    if let Some(at_pos) = username.find('@') {
                        let local_part = &username[..at_pos];
                        // Clean the local part to make it suitable for Minecraft
                        let clean = local_part.chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .take(16)
                            .collect::<String>();
                        if clean.is_empty() {
                            format!("User_{}", &username.chars().filter(|c| c.is_alphanumeric()).take(8).collect::<String>())
                        } else {
                            clean
                        }
                    } else {
                        // Not an email, clean the username
                        let clean = username.chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .take(16)
                            .collect::<String>();
                        if clean.is_empty() { "Player".to_string() } else { clean }
                    }
                }
            }
            ql_instances::auth::AccountType::Microsoft => {
                // For Microsoft accounts, prefer nice_username (gamertag) if valid
                if nice_username.chars().all(|c| c.is_alphanumeric() || c == '_') && nice_username.len() <= 16 {
                    nice_username.to_string()
                } else {
                    // Fallback to cleaning the username
                    let clean = username.chars()
                        .filter(|c| c.is_alphanumeric() || *c == '_')
                        .take(16)
                        .collect::<String>();
                    if clean.is_empty() { "Player".to_string() } else { clean }
                }
            }
        }
    }

    /// Generate a clean username for a selected account without full auth data
    fn get_clean_username_for_selected_account(&self, account: &Account) -> String {
        // For offline accounts, use the username directly (it's already what the user specified)
        if account.account_type == "Offline" {
            // Clean the username to ensure it's valid for Minecraft
            let clean = account.username.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .take(16)
                .collect::<String>();
            if clean.is_empty() { "Player".to_string() } else { clean }
        } else {
            // Try to get the nice_username from config if available
            if let Ok(config) = LauncherConfig::load_s() {
                if let Some(config_accounts) = config.accounts {
                    if let Some(config_account) = config_accounts.get(&account.username) {
                        if let Some(nice_username) = &config_account.username_nice {
                            // Use nice_username if it's clean
                            if nice_username.chars().all(|c| c.is_alphanumeric() || c == '_') && nice_username.len() <= 16 {
                                return nice_username.clone();
                            }
                        }
                    }
                }
            }

            // Fallback to cleaning the account username
            match account.account_type.as_str() {
                "ElyBy" | "LittleSkin" => {
                    // For ElyBy/LittleSkin, try to extract from email
                    if let Some(at_pos) = account.username.find('@') {
                        let local_part = &account.username[..at_pos];
                        let clean = local_part.chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .take(16)
                            .collect::<String>();
                        if clean.is_empty() {
                            format!("User_{}", &account.username.chars().filter(|c| c.is_alphanumeric()).take(8).collect::<String>())
                        } else {
                            clean
                        }
                    } else {
                        let clean = account.username.chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .take(16)
                            .collect::<String>();
                        if clean.is_empty() { "Player".to_string() } else { clean }
                    }
                }
                _ => {
                    // For Microsoft and others, just clean the username
                    let clean = account.username.chars()
                        .filter(|c| c.is_alphanumeric() || *c == '_')
                        .take(16)
                        .collect::<String>();
                    if clean.is_empty() { "Player".to_string() } else { clean }
                }
            }
        }
    }

    /// Add a log line to game logs
    pub fn add_log(&mut self, log_line: String) {
        self.game_logs.push(log_line);
        // Keep only last 1000 lines to prevent memory issues
        if self.game_logs.len() > 1000 {
            self.game_logs.remove(0);
        }
    }

    /// Clear game logs
    pub fn clear_logs(&mut self) {
        self.game_logs.clear();
    }

    /// Check if forced refresh is needed and reset the flag
    pub fn check_and_reset_forced_refresh(&mut self) -> bool {
        let needs_refresh = self.needs_forced_refresh;
        self.needs_forced_refresh = false;
        needs_refresh
    }
}
