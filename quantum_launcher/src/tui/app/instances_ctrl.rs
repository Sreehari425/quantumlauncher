// Instances controls: refresh (sync and async)

use std::collections::HashSet;
use crate::tui::app::{App, Instance};

impl App {
    /// Start async refresh of instances
    pub fn start_refresh(&mut self) {
        if let Some(sender) = &self.auth_sender {
            let sender_clone = sender.clone();
            let _ = sender.send(crate::tui::AuthEvent::RefreshStarted);
            tokio::spawn(async move {
                match crate::state::get_entries("instances".to_owned(), false).await {
                    Ok((instance_names, _)) => {
                        let mut instance_data = Vec::new();
                        for name in instance_names {
                            let instance_dir = match ql_core::file_utils::get_launcher_dir() {
                                Ok(launcher_dir) => launcher_dir.join("instances").join(&name),
                                Err(_) => continue,
                            };
                            let loader = match ql_core::json::InstanceConfigJson::read_from_dir(&instance_dir).await {
                                Ok(cfg) => cfg.mod_type,
                                Err(_) => "Vanilla".to_string(),
                            };
                            let version = match ql_core::json::VersionDetails::load_from_path(&instance_dir).await {
                                Ok(details) => details.id,
                                Err(_) => "Unknown".to_string(),
                            };
                            instance_data.push((name, version, loader));
                        }
                        let _ = sender_clone.send(crate::tui::AuthEvent::RefreshData { instances: instance_data });
                        let _ = sender_clone.send(crate::tui::AuthEvent::RefreshCompleted);
                    }
                    Err(_) => {
                        let _ = sender_clone.send(crate::tui::AuthEvent::RefreshCompleted);
                    }
                }
            });
        }
    }

    /// Synchronous refresh used on startup
    pub fn refresh(&mut self) {
        use std::path::PathBuf;
        use crate::state::get_entries;
        use ql_core::json::{InstanceConfigJson, VersionDetails};
        use ql_core::file_utils;

        self.is_loading = true;
        self.status_message = "Refreshing...".to_string();

        let running_instances: HashSet<String> = self.instances.iter().filter(|i| i.is_running).map(|i| i.name.clone()).collect();

        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(get_entries("instances".to_owned(), false)) {
                    Ok((instance_names, _)) => {
                        self.instances.clear();
                        let launcher_dir = match file_utils::get_launcher_dir() { Ok(dir) => dir, Err(_) => PathBuf::from(".config/QuantumLauncher") };
                        let instances_dir = launcher_dir.join("instances");
                        for name in instance_names {
                            let instance_dir = instances_dir.join(&name);
                            let loader = match rt.block_on(InstanceConfigJson::read_from_dir(&instance_dir)) { Ok(cfg) => cfg.mod_type, Err(_) => "Vanilla".to_string() };
                            let version = match rt.block_on(VersionDetails::load_from_path(&instance_dir)) { Ok(details) => details.id, Err(_) => "Unknown".to_string() };
                            self.instances.push(Instance { name: name.clone(), version, loader, is_running: running_instances.contains(&name) });
                        }
                        self.status_message = format!("Loaded {} instances", self.instances.len());
                    }
                    Err(e) => { self.status_message = format!("Failed to load instances: {}", e); }
                }
                if self.available_versions.is_empty() {
                    match rt.block_on(ql_instances::list_versions()) {
                        Ok(versions) => { self.available_versions = versions; self.update_filtered_versions(); }
                        Err(e) => { self.status_message = format!("Failed to load versions: {}", e); }
                    }
                }
            }
            Err(e) => { self.status_message = format!("Failed to create runtime: {}", e); }
        }

        self.is_loading = false;
    }
}
