// QuantumLauncher TUI - Application State 

use std::{error::Error, fmt, collections::{HashSet, HashMap}};
use ql_core::ListEntry;
use crate::config::LauncherConfig;
use crate::state::{ClientProcess};
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
	InstanceSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddAccountFieldFocus {
	Username,
	Password,
	Otp,
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
			TabId::InstanceSettings => write!(f, "Instance Settings"),
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
	// Version search/filtering (Create tab)
	pub version_search_active: bool,
	pub version_search_query: String,
	pub filtered_versions: Vec<ListEntry>,
	pub selected_filtered_version: usize,
	// Version type filters
	pub filter_release: bool,
	pub filter_snapshot: bool,
	pub filter_beta: bool,
	pub filter_alpha: bool,
	pub new_instance_name: String,
	pub is_editing_name: bool,
	pub download_assets: bool,
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
	// Logs view state
	pub logs_offset: usize,            // number of lines scrolled up from the bottom (0 = bottom)
	pub logs_visible_lines: usize,     // lines that fit in viewport (updated by renderer)
	pub logs_auto_follow: bool,        // auto follow new logs when at bottom
	// Terminal refresh tracking
	pub needs_forced_refresh: bool,
	// Instance settings fields
	pub instance_settings_instance: Option<usize>, // Index of the instance being viewed in settings
	pub instance_settings_tab: InstanceSettingsTab, // Current tab in instance settings
	pub instance_settings_selected: usize, // Selected item in instance settings
	// Process tracking for kill functionality
	pub client_processes: HashMap<String, ClientProcess>, // Track running instances
	pub show_delete_confirm: bool, // Show confirmation popup for instance deletion
	// About page state
	pub about_selected: usize, // Left menu selection index
	pub about_scroll: u16,     // Right content scroll offset
	// Settings → Licenses submenu state
	pub settings_focus: SettingsFocus, // Which pane is focused in Settings
	pub license_selected: usize,       // Selected license in the Licenses submenu
	// Per-instance logs (instance name -> recent lines)
	pub instance_logs: HashMap<String, Vec<String>>,
	// Per-instance logs view state
	pub instance_logs_offset: usize,        // number of lines scrolled up from bottom (0 = bottom)
	pub instance_logs_visible_lines: usize, // lines that fit in viewport (set by renderer)
	pub instance_logs_auto_follow: bool,    // follow new lines when at bottom
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceSettingsTab {
	Overview,
	Mod,
	Setting,
	Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
	Left,
	Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCategory {
	Release,
	Snapshot,
	Beta,
	Alpha,
}

impl App {
	pub fn new() -> Self {
	let mut app = Self {
			current_tab: TabId::Instances,
			instances: Vec::new(),
			selected_instance: 0,
			available_versions: Vec::new(),
			version_search_active: false,
			version_search_query: String::new(),
			filtered_versions: Vec::new(),
			selected_filtered_version: 0,
			filter_release: true,
			filter_snapshot: true,
			filter_beta: true,
			filter_alpha: true,
			new_instance_name: String::new(),
			is_editing_name: false,
			download_assets: true, // Default to true like in Iced UI
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
			logs_offset: 0,
			logs_visible_lines: 20,
			logs_auto_follow: true,
			needs_forced_refresh: false,
			// Initialize instance settings fields
			instance_settings_instance: None,
			instance_settings_tab: InstanceSettingsTab::Overview,
			instance_settings_selected: 0,
			// Initialize client process tracking
			client_processes: HashMap::new(),
			show_delete_confirm: false,
			// About
			about_selected: 0,
			about_scroll: 0,
			settings_focus: SettingsFocus::Left,
			license_selected: 0,
			instance_logs: HashMap::new(),
			instance_logs_offset: 0,
			instance_logs_visible_lines: 20,
			instance_logs_auto_follow: true,
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
			TabId::InstanceSettings => TabId::Instances,
		};
	}

	pub fn previous_tab(&mut self) {
		self.current_tab = match self.current_tab {
			TabId::Instances => TabId::Logs,
			TabId::Create => TabId::Instances,
			TabId::Settings => TabId::Create,
			TabId::Accounts => TabId::Settings,
			TabId::Logs => TabId::Accounts,
			TabId::InstanceSettings => TabId::Instances,
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
				if !self.filtered_versions.is_empty() {
					self.selected_filtered_version = (self.selected_filtered_version + 1) % self.filtered_versions.len();
				}
			}
			TabId::Accounts => {
				if !self.accounts.is_empty() {
					self.selected_account = (self.selected_account + 1) % self.accounts.len();
				}
			}
			TabId::Settings => {
				// Settings left menu (General, Java, UI/Theme, Launch, Licenses, About = 6 items)
				if self.settings_focus == SettingsFocus::Middle && self.about_selected == Self::licenses_menu_index() {
					// license_selected uses 0 = About, 1..=N = licenses; allow cycling through About + N licenses
					let count = Self::licenses().len() + 1; // +1 for the About entry
					if count > 0 {
						self.license_selected = (self.license_selected + 1) % count;
					}
				} else {
					self.about_selected = (self.about_selected + 1) % 6;
					self.about_scroll = 0;
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
				if !self.filtered_versions.is_empty() && self.selected_filtered_version > 0 {
					self.selected_filtered_version -= 1;
				} else if !self.filtered_versions.is_empty() {
					self.selected_filtered_version = self.filtered_versions.len() - 1;
				}
			}
			TabId::Accounts => {
				if !self.accounts.is_empty() && self.selected_account > 0 {
					self.selected_account -= 1;
				} else if !self.accounts.is_empty() {
					self.selected_account = self.accounts.len() - 1;
				}
			}
			TabId::Settings => {
				if self.settings_focus == SettingsFocus::Middle && self.about_selected == Self::licenses_menu_index() {
					// Match next_item behavior: include About as an option (0), so total = licenses.len() + 1
					let count = Self::licenses().len() + 1;
					if count > 0 {
						if self.license_selected > 0 { self.license_selected -= 1; } else { self.license_selected = count - 1; }
					}
				} else {
					if self.about_selected > 0 { self.about_selected -= 1; } else { self.about_selected = 5; }
					self.about_scroll = 0;
				}
			}
			_ => {}
		}
	}

	// Index of the combined "About & Licenses" entry in the left settings menu
	pub const fn licenses_menu_index() -> usize { 4 }

	pub fn licenses() -> &'static [(&'static str, &'static [&'static str])] {
		// Display labels chosen to match the GUI-style menu
		&[
			// QuantumLauncher project license (GPLv3)
			(
				"QuantumLauncher (GPLv3)",
				&[
					"LICENSE",
					"../LICENSE",
					"../../LICENSE",
				],
			),
			// Forge installer (Apache-2.0)
			("Forge Installer", &["assets/licenses/APACHE_2.txt", "../assets/licenses/APACHE_2.txt", "../../assets/licenses/APACHE_2.txt"]),
			// Fonts packaged under OFL
			("Fonts (Inter/Jetbrains Mono)", &["assets/licenses/OFL.txt", "../assets/licenses/OFL.txt", "../../assets/licenses/OFL.txt"]),
			// Password asterisks asset license (use CC BY-SA where provided)
			("Password Asterisks Font", &["assets/licenses/CC_BY_SA_3_0.txt", "../assets/licenses/CC_BY_SA_3_0.txt", "../../assets/licenses/CC_BY_SA_3_0.txt"]),
			// LWJGL license
			("LWJGL", &["assets/licenses/LWJGL.txt", "../assets/licenses/LWJGL.txt", "../../assets/licenses/LWJGL.txt"]),
		]
	}

	/// Compile-time embedded license text fallbacks (mirrors the Iced UI behavior)
	/// Index corresponds to `Self::licenses()` ordering
	pub fn license_fallback_content(index: usize) -> Option<&'static str> {
		match index {
			// QuantumLauncher GPLv3
			0 => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../LICENSE"))),
			// Forge Installer (Apache 2.0)
			1 => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/licenses/APACHE_2.txt"))),
			// Fonts (OFL)
			2 => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/licenses/OFL.txt"))),
			// Password Asterisks Font (CC BY-SA 3.0)
			3 => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/licenses/CC_BY_SA_3_0.txt"))),
			// LWJGL
			4 => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/licenses/LWJGL.txt"))),
			_ => None,
		}
	}

	// Removed load_license_text: Settings tab reads files/fallbacks directly


	pub fn select_item(&mut self) {
		match self.current_tab {
			TabId::Instances => {
				// Open instance settings page instead of launching directly
				self.instance_settings_instance = Some(self.selected_instance);
				self.instance_settings_tab = InstanceSettingsTab::Overview;
				self.instance_settings_selected = 0;
				self.current_tab = TabId::InstanceSettings;
				self.status_message = format!("Opened settings for instance: {}", self.instances[self.selected_instance].name);
			}
			TabId::Create => {
				// Create tab should not use select_item - use create_instance instead
				self.status_message = "Use Enter to create instance or Ctrl+N to edit name".to_string();
			}
			_ => {}
		}
	}



	/// Start async refresh of instances
	pub fn start_refresh(&mut self) {
		if let Some(sender) = &self.auth_sender {
			let sender_clone = sender.clone();
            
			// Send refresh started event
			let _ = sender.send(crate::tui::AuthEvent::RefreshStarted);
            
			// Spawn refresh task using the existing async infrastructure
			tokio::spawn(async move {
				match crate::state::get_entries("instances".to_owned(), false).await {
					Ok((instance_names, _)) => {
						let mut instance_data = Vec::new();
                        
						// Gather data for each instance
						for name in instance_names {
							let instance_dir = match ql_core::file_utils::get_launcher_dir() {
								Ok(launcher_dir) => launcher_dir.join("instances").join(&name),
								Err(_) => continue,
							};
                            
							// Get loader info from config.json
							let loader = match ql_core::json::InstanceConfigJson::read_from_dir(&instance_dir).await {
								Ok(cfg) => cfg.mod_type,
								Err(_) => "Vanilla".to_string(),
							};
                            
							// Get version info from details.json  
							let version = match ql_core::json::VersionDetails::load_from_path(&instance_dir).await {
								Ok(details) => details.id,
								Err(_) => "Unknown".to_string(),
							};
                            
							instance_data.push((name, version, loader));
						}
                        
						// Send the instance data back
						let _ = sender_clone.send(crate::tui::AuthEvent::RefreshData { instances: instance_data });
						let _ = sender_clone.send(crate::tui::AuthEvent::RefreshCompleted);
					}
					Err(_) => {
						// Still signal completion even on error
						let _ = sender_clone.send(crate::tui::AuthEvent::RefreshCompleted);
					}
				}
			});
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
							// Initialize filtered list using current filters
							self.update_filtered_versions();
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



	pub fn toggle_help_popup(&mut self) {
		self.show_help_popup = !self.show_help_popup;
	}


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
				self.status_message = format!("LAUNCHING: {}...", instance_name);
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
                
				// Store the process in client_processes for kill functionality
				let client_process = ClientProcess {
					child: child.clone(),
					receiver: None, // No log receiver in TUI version
				};
				self.client_processes.insert(instance_name.clone(), client_process);
                
				// NEW: Stream game's stdout/stderr into the core logger and per-instance buffer so Logs UIs update live
				{
					// Take stdout/stderr handles once and spawn readers
					let (stdout, stderr) = {
						let mut guard = child.lock().unwrap();
						(guard.stdout.take(), guard.stderr.take())
					};

					if let Some(mut out) = stdout {
						let inst_name = instance_name.clone();
						let ui_tx = self.auth_sender.clone();
						tokio::spawn(async move {
							use tokio::io::{AsyncBufReadExt, BufReader};
							let mut lines = BufReader::new(&mut out).lines();
							while let Ok(Some(line)) = lines.next_line().await {
								// Trim and append newline to match other log entries
								let mut s = line;
								if !s.ends_with('\n') { s.push('\n'); }
								if let Some(tx) = &ui_tx { let _ = tx.send(crate::tui::AuthEvent::InstanceLogLine { instance_name: inst_name.clone(), line: s.clone() }); }
							}
						});
					}
					if let Some(mut err) = stderr {
						let inst_name = instance_name.clone();
						let ui_tx = self.auth_sender.clone();
						tokio::spawn(async move {
							use tokio::io::{AsyncBufReadExt, BufReader};
							let mut lines = BufReader::new(&mut err).lines();
							while let Ok(Some(line)) = lines.next_line().await {
								let mut s = line;
								if !s.ends_with('\n') { s.push('\n'); }
								if let Some(tx) = &ui_tx { let _ = tx.send(crate::tui::AuthEvent::InstanceLogLine { instance_name: inst_name.clone(), line: s.clone() }); }
							}
						});
					}
				}
                
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
				// Remove the process from tracking
				self.client_processes.remove(&instance_name);
                
				// Mark the instance as stopped
				if let Some(instance) = self.instances.iter_mut().find(|i| i.name == instance_name) {
					instance.is_running = false;
				}
				let msg = format!("🛑 {} has stopped", instance_name);
				self.status_message = msg.clone();
				self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
			}
			crate::tui::AuthEvent::InstanceLogLine { instance_name, line } => {
				// Update instance-specific buffer (limit to 2000 lines per instance)
				let buf = self.instance_logs.entry(instance_name).or_default();
				let ln = line.trim_end_matches('\n').to_string();
				buf.push(ln);
				if buf.len() > 2000 { let excess = buf.len() - 2000; buf.drain(0..excess); }
				// Auto-follow only when viewing instance logs and at bottom
				if self.current_tab == crate::tui::app::TabId::InstanceSettings
					&& self.instance_settings_tab == crate::tui::app::InstanceSettingsTab::Logs
					&& self.instance_logs_auto_follow
				{
					self.instance_logs_offset = 0;
				}
			}
			crate::tui::AuthEvent::InstanceCreateStarted(instance_name) => {
				self.status_message = format!("🔨 Creating instance '{}'... This may take a while", instance_name);
				self.is_loading = true;
				self.add_log(format!("[{}] Started creating instance: {}", 
					chrono::Local::now().format("%H:%M:%S"), instance_name));
			}
			crate::tui::AuthEvent::InstanceCreateProgress { instance_name, message } => {
				self.status_message = format!("🔨 Creating '{}': {}", instance_name, message);
				self.add_log(format!("[{}] Instance '{}': {}", 
					chrono::Local::now().format("%H:%M:%S"), instance_name, message));
			}
			crate::tui::AuthEvent::InstanceCreateSuccess { instance_name } => {
				self.is_loading = false;
				let msg = format!("✅ Successfully created instance '{}'", instance_name);
				self.status_message = msg.clone();
				self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
                
				// Switch to instances tab (don't auto-refresh to avoid runtime conflict)
				self.current_tab = TabId::Instances;
				self.status_message = format!("✅ Successfully created instance '{}'. Press F5 to refresh instances list.", instance_name);
			}
			crate::tui::AuthEvent::InstanceCreateError { instance_name, error } => {
				self.is_loading = false;
				let msg = format!("❌ Failed to create instance '{}': {}", instance_name, error);
				self.status_message = msg.clone();
				self.add_log(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
			}
			crate::tui::AuthEvent::RefreshStarted => {
				self.status_message = "🔄 Refreshing instances...".to_string();
				self.is_loading = true;
				self.add_log(format!("[{}] Started refreshing instances", 
					chrono::Local::now().format("%H:%M:%S")));
			}
			crate::tui::AuthEvent::RefreshCompleted => {
				self.is_loading = false;
				self.status_message = "✅ Instances refreshed successfully".to_string();
				self.add_log(format!("[{}] Instances refresh completed", 
					chrono::Local::now().format("%H:%M:%S")));
			}
			crate::tui::AuthEvent::RefreshData { instances } => {
				// Preserve running status
				let running_instances: HashSet<String> = self.instances
					.iter()
					.filter(|i| i.is_running)
					.map(|i| i.name.clone())
					.collect();
                
				// Update instances with new data
				self.instances.clear();
				for (name, version, loader) in instances {
					self.instances.push(Instance {
						name: name.clone(),
						version,
						loader,
						is_running: running_instances.contains(&name),
					});
				}
                
				// Ensure selected instance is still valid
				if self.selected_instance >= self.instances.len() && !self.instances.is_empty() {
					self.selected_instance = self.instances.len() - 1;
				} else if self.instances.is_empty() {
					self.selected_instance = 0;
				}
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

	/// Save config synchronously (helper for TUI)
	pub(crate) fn save_config_sync(&self, config: &LauncherConfig) -> Result<(), String> {
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
    


	/// Add a log line to game logs
	pub fn add_log(&mut self, log_line: String) {
	// Push to local buffer for immediate rendering
	self.game_logs.push(log_line.clone());
	// Mirror into core in-memory logger so external readers (and future sessions) see it
	let mut s = log_line;
	if !s.ends_with('\n') { s.push('\n'); }
	ql_core::print::print_to_storage(&s, ql_core::print::LogType::Info);
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

