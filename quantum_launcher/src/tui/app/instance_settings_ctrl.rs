// Instance Settings controls and actions (navigation, play/kill/open/delete)

use crate::tui::app::{App, InstanceSettingsTab, TabId};
use crate::tui::app::InstanceSettingsPage;

impl App {
    /// Navigate to next instance settings tab
    pub fn next_instance_settings_tab(&mut self) {
        self.instance_settings_tab = match self.instance_settings_tab {
            InstanceSettingsTab::Overview => InstanceSettingsTab::Mod,
            InstanceSettingsTab::Mod => InstanceSettingsTab::Setting,
            InstanceSettingsTab::Setting => InstanceSettingsTab::Logs,
            InstanceSettingsTab::Logs => InstanceSettingsTab::Overview,
        };
        self.instance_settings_selected = 0; // Reset selection when switching tabs
    }

    /// Navigate to previous instance settings tab
    pub fn prev_instance_settings_tab(&mut self) {
        self.instance_settings_tab = match self.instance_settings_tab {
            InstanceSettingsTab::Overview => InstanceSettingsTab::Logs,
            InstanceSettingsTab::Mod => InstanceSettingsTab::Overview,
            InstanceSettingsTab::Setting => InstanceSettingsTab::Mod,
            InstanceSettingsTab::Logs => InstanceSettingsTab::Setting,
        };
        self.instance_settings_selected = 0; // Reset selection when switching tabs
    }

    /// Select item in instance settings
    pub fn select_instance_settings_item(&mut self) {
        if let Some(instance_idx) = self.instance_settings_instance {
            if let Some(instance) = self.instances.get(instance_idx) {
                let instance_name = instance.name.clone();
                let instance_running = instance.is_running;
                match self.instance_settings_tab {
                    InstanceSettingsTab::Overview => match self.instance_settings_selected {
                        0 => {
                            // Play button
                            if instance_running {
                                self.status_message = format!("❌ Instance '{}' is already running", instance_name);
                            } else {
                                self.launch_instance(&instance_name);
                                self.current_tab = TabId::Instances; // Return to instances after launching
                            }
                        }
                        1 => {
                            // Kill button
                            self.kill_instance(&instance_name);
                        }
                        2 => {
                            // Open Folder button
                            self.open_instance_folder(&instance_name);
                        }
                        _ => {}
                    },
                    InstanceSettingsTab::Mod => {
                        self.status_message = "Mod management feature coming soon".to_string();
                    }
                    InstanceSettingsTab::Setting => match self.instance_settings_selected {
                        0 => {
                            // Rename Instance -> open inline popup for renaming
                            if let Some(idx) = self.instance_settings_instance {
                                self.is_renaming_instance = true;
                                self.rename_input = self.instances[idx].name.clone();
                                self.status_message = "Editing instance name. Type new name and press Enter to apply, Esc to cancel.".to_string();
                            }
                        }
                        1 => {
                            // Java Settings -> go to Java subpage (with placeholders and Memory)
                            self.instance_settings_page = InstanceSettingsPage::Java;
                            self.preload_memory_summary();
                            self.instance_settings_selected = 0; // selection within Java page
                            self.status_message = "Opened Java settings".to_string();
                        }
                        2 => {
                            // Launch Options
                            self.instance_settings_page = InstanceSettingsPage::Launch;
                            self.status_message = "Opened Launch options".to_string();
                        }
                        3 => {
                            // Delete Instance
                            self.show_delete_confirm = true;
                        }
                        _ => {}
                    },
                    InstanceSettingsTab::Logs => {
                        self.status_message = "Instance-specific logs coming soon".to_string();
                    }
                }
            }
        }
    }

    /// Navigate items in instance settings
    pub fn navigate_instance_settings(&mut self, direction: i32) {
        let max_items = match self.instance_settings_tab {
            InstanceSettingsTab::Overview => 3, // Play, Kill, and Open Folder buttons
            InstanceSettingsTab::Mod => 1,
            InstanceSettingsTab::Setting => match self.instance_settings_page { InstanceSettingsPage::List => 4, InstanceSettingsPage::Java => 6, InstanceSettingsPage::Launch => 5 },
            InstanceSettingsTab::Logs => 1,     // Logs message
        };

        if max_items > 1 {
            self.instance_settings_selected =
                (self.instance_settings_selected as i32 + direction).rem_euclid(max_items) as usize;
        }
    }

    /// Handle Enter inside Java subpage list
    pub fn select_in_java_page(&mut self) {
        match self.instance_settings_selected {
            0 => { self.status_message = "(placeholder) Custom Java executable".to_string(); }
            1 => { self.status_message = "(placeholder) Java arguments mode".to_string(); }
            2 => { self.open_java_args_edit(); }
            3 => { self.status_message = "(placeholder) Pre-launch prefix mode".to_string(); }
            4 => { self.status_message = "(placeholder) Pre-launch prefix commands (global)".to_string(); }
            5 => { // Memory
                self.open_memory_edit();
            }
            _ => {}
        }
    }

    /// Handle Enter inside Launch subpage list
    pub fn select_in_launch_page(&mut self) {
        match self.instance_settings_selected {
            0 => { self.open_game_args_edit(); }
            _ => {}
        }
    }

    /// Open popup to edit Java arguments (text based)
    pub fn open_java_args_edit(&mut self) {
        self.open_args_edit(true);
    }

    /// Open popup to edit Game arguments (text based)
    pub fn open_game_args_edit(&mut self) {
        self.open_args_edit(false);
    }

    fn open_args_edit(&mut self, java: bool) {
        use crate::tui::app::ArgsEditKind;
        use ql_core::{file_utils, json::InstanceConfigJson};
        use std::path::PathBuf;
        let Some(idx) = self.instance_settings_instance else { return; };
        let Some(inst) = self.instances.get(idx) else { return; };
        let instance_name = inst.name.clone();
        // Load current args from config.json
        let text = match file_utils::get_launcher_dir() {
            Ok(dir) => {
                let mut p = PathBuf::from(dir);
                p.push("instances");
                p.push(&instance_name);
                match std::fs::read_to_string(p.join("config.json")) {
                    Ok(s) => match serde_json::from_str::<InstanceConfigJson>(&s) {
                        Ok(cfg) => {
                            let vec = if java { cfg.java_args.unwrap_or_default() } else { cfg.game_args.unwrap_or_default() };
                            // Join with commas; quote if spaces or commas present
                            vec.iter().map(|a| {
                                if a.is_empty() { "\"\"".to_string() }
                                else if a.chars().any(|c| c.is_whitespace() || c == ',') { format!("\"{}\"", a.replace('"', "\\\"")) }
                                else { a.clone() }
                            }).collect::<Vec<_>>().join(",")
                        }
                        Err(_) => String::new(),
                    },
                    Err(_) => String::new(),
                }
            }
            Err(_) => String::new(),
        };
        self.is_editing_args = true;
        self.args_edit_input = text;
        self.args_edit_kind = if java { ArgsEditKind::Java } else { ArgsEditKind::Game };
        self.status_message = if java { "Editing Java arguments".to_string() } else { "Editing Game arguments".to_string() };
    }

    /// Apply rename from popup buffer to disk and in-memory list
    pub fn apply_rename_instance(&mut self) {
        if !self.is_renaming_instance { return; }
        let Some(idx) = self.instance_settings_instance else { return; };
        let old_name = self.instances[idx].name.clone();
        // Sanitize similar to iced implementation
        let mut disallowed = vec!['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\'', '\0', '\u{7F}'];
        disallowed.extend('\u{1}'..='\u{1F}');

        let mut new_name: String = self.rename_input.clone();
        new_name.retain(|c| !disallowed.contains(&c));
        let new_name = new_name.trim().to_string();

        if new_name.is_empty() {
            self.status_message = "❌ New name is empty or invalid".to_string();
            return; // keep popup open for correction
        }
        if new_name == old_name { 
            self.is_renaming_instance = false; 
            self.status_message = "No changes to apply".to_string();
            return; 
        }

        // Refuse rename if running
        if self.client_processes.contains_key(&old_name) {
            self.status_message = "❌ Cannot rename a running instance".to_string();
            return;
        }

        match ql_core::file_utils::get_launcher_dir() {
            Ok(launcher_dir) => {
                let instances_dir = launcher_dir.join("instances");
                let old_path = instances_dir.join(&old_name);
                let new_path = instances_dir.join(&new_name);
                if new_path.exists() {
                    self.status_message = format!("❌ An instance named '{}' already exists", new_name);
                    return;
                }
                match std::fs::rename(&old_path, &new_path) {
                    Ok(_) => {
                        // Update in-memory list
                        if let Some(inst) = self.instances.get_mut(idx) { inst.name = new_name.clone(); }
                        // Also update selected_instance if needed
                        if self.selected_instance == idx { self.selected_instance = idx; }
                        // Close popup
                        self.is_renaming_instance = false;
                        self.status_message = format!("✅ Renamed instance '{}' → '{}'", old_name, new_name);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Failed to rename: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("❌ Failed to get launcher directory: {}", e);
            }
        }
    }

    /// Cancel rename flow
    pub fn cancel_rename_instance(&mut self) {
        self.is_renaming_instance = false;
        self.status_message = "Rename cancelled".to_string();
    }

    /// Kill a running instance
    pub fn kill_instance(&mut self, instance_name: &str) {
        if let Some(process) = self.client_processes.remove(instance_name) {
            self.status_message = format!("🔪 Terminating instance: {}", instance_name);

            // Spawn a task to kill the process
            if let Some(sender) = &self.auth_sender {
                let sender_clone = sender.clone();
                let instance_name_clone = instance_name.to_string();

                tokio::spawn(async move {
                    // same logic as iced UI
                    let result = {
                        let mut child = process.child.lock().unwrap();
                        child.start_kill()
                    };

                    if let Err(e) = result {
                        eprintln!("Failed to kill process gracefully: {}", e);
                    }

                    //send LaunchEnded to update the UI
                    let _ = sender_clone.send(crate::tui::AuthEvent::LaunchEnded(instance_name_clone));
                });
            }
        } else {
            self.status_message = format!("❌ Instance {} is not running", instance_name);
        }
    }

    /// Open instance folder in file explorer
    pub fn open_instance_folder(&mut self, instance_name: &str) {
        match ql_core::file_utils::get_launcher_dir() {
            Ok(launcher_dir) => {
                let instance_path = launcher_dir.join("instances").join(instance_name);

                if instance_path.exists() {
                    self.status_message = format!("📂 Opening folder for instance: {}", instance_name);
                    ql_core::open_file_explorer(&instance_path);
                } else {
                    self.status_message = format!("❌ Instance folder not found: {}", instance_name);
                }
            }
            Err(e) => {
                self.status_message = format!("❌ Failed to get launcher directory: {}", e);
            }
        }
    }

    /// Delete an instance permanently
    pub fn delete_instance(&mut self, instance_name: &str) {
        //check if the instance is running and refuse deletion if it is
        if self.client_processes.contains_key(instance_name) {
            self.status_message = format!(
                "❌ Cannot delete '{}': instance is currently running. Stop it first.",
                instance_name
            );
            return;
        }

        match ql_core::file_utils::get_launcher_dir() {
            Ok(launcher_dir) => {
                let instance_path = launcher_dir.join("instances").join(instance_name);

                if instance_path.exists() {
                    // Try to delete the instance directory
                    if let Err(e) = std::fs::remove_dir_all(&instance_path) {
                        self.status_message =
                            format!("❌ Failed to delete instance '{}': {}", instance_name, e);
                    } else {
                        // Remove the instance from the list
                        self.instances.retain(|instance| instance.name != instance_name);

                        // Reset selection if needed
                        if self.selected_instance >= self.instances.len() && !self.instances.is_empty() {
                            self.selected_instance = self.instances.len() - 1;
                        } else if self.instances.is_empty() {
                            self.selected_instance = 0;
                        }

                        // Return to instances tab
                        self.current_tab = TabId::Instances;
                        self.instance_settings_instance = None;

                        self.status_message =
                            format!("DELETED: Successfully removed instance {}", instance_name);
                    }
                } else {
                    self.status_message =
                        format!("ERROR: Instance folder not found: {}", instance_name);
                }
            }
            Err(e) => {
                self.status_message = format!("ERROR: Failed to get launcher directory: {}", e);
            }
        }
    }
}
