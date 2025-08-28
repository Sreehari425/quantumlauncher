/*
QuantumLauncher TUI - Application State
Copyright (C) 2024  Mrmayman & Contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::{error::Error, fmt};
use ql_core::ListEntry;
use crate::state::get_entries;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TabId {
    Instances,
    Create,
    Settings,
    Accounts,
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
            status_message: "Welcome to QuantumLauncher TUI! Press '?' for help, 'q' to quit.".to_string(),
            should_quit: false,
            is_loading: false,
        };
        
        // Load instances on startup
        app.refresh();
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
            _ => {}
        }
    }

    pub fn previous_item(&mut self) {
        match self.current_tab {
            TabId::Instances => {
                if !self.instances.is_empty() {
                    self.selected_instance = if self.selected_instance == 0 {
                        self.instances.len() - 1
                    } else {
                        self.selected_instance - 1
                    };
                }
            }
            TabId::Create => {
                if !self.available_versions.is_empty() {
                    self.selected_version = if self.selected_version == 0 {
                        self.available_versions.len() - 1
                    } else {
                        self.selected_version - 1
                    };
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
        self.is_loading = true;
        self.status_message = "Refreshing...".to_string();
        
        // Load instances
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(get_entries("instances".to_owned(), false)) {
                    Ok((instance_names, _)) => {
                        self.instances.clear();
                        for name in instance_names {
                            // For now, create dummy data - in a real implementation,
                            // you'd read the actual instance details
                            self.instances.push(Instance {
                                name: name.clone(),
                                version: "Unknown".to_string(),
                                loader: "Vanilla".to_string(),
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

    pub fn toggle_help_popup(&mut self) {
        self.show_help_popup = !self.show_help_popup;
    }
}
