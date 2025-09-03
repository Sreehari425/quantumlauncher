// Instance Settings controls and actions (navigation, play/kill/open/delete)

use crate::tui::app::{App, InstanceSettingsTab, TabId};

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
                            // Rename Instance
                            self.status_message = "Rename instance feature coming soon".to_string();
                        }
                        1 => {
                            // Java Settings
                            self.status_message = "Java configuration feature coming soon".to_string();
                        }
                        2 => {
                            // Launch Options
                            self.status_message = "Launch options configuration coming soon".to_string();
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
            InstanceSettingsTab::Setting => 4,  // Rename, Java Settings, Launch Options, Delete
            InstanceSettingsTab::Logs => 1,     // Logs message
        };

        if max_items > 1 {
            self.instance_settings_selected =
                (self.instance_settings_selected as i32 + direction).rem_euclid(max_items) as usize;
        }
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
