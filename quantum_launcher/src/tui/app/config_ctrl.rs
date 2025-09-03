// Config helpers: saving config synchronously for TUI flows

use crate::config::LauncherConfig;
use crate::tui::app::App;

impl App {
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
}
