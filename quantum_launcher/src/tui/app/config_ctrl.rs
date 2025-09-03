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

    /// Open popup to edit global Java arguments (LauncherConfig.extra_java_args)
    pub fn open_global_java_args_edit(&mut self) {
        use crate::tui::app::ArgsEditKind;
        let cfg = match LauncherConfig::load_s() {
            Ok(c) => c,
            Err(e) => { self.status_message = format!("❌ Failed to load global config: {}", e); return; }
        };
        let vec = cfg.extra_java_args.unwrap_or_default();
        let text = vec.iter().map(|a| {
            if a.is_empty() { "\"\"".to_string() }
            else if a.chars().any(|c| c.is_whitespace() || c == ',') { format!("\"{}\"", a.replace('"', "\\\"")) }
            else { a.clone() }
        }).collect::<Vec<_>>().join(",");

        self.is_editing_args = true;
        self.args_edit_input = text;
        self.args_edit_kind = ArgsEditKind::GlobalJava;
        self.status_message = "Editing global Java arguments".to_string();
    }

    /// Save args popup buffer into global LauncherConfig.extra_java_args
    pub fn apply_global_java_args_edit(&mut self) {
        if !self.is_editing_args { return; }
        if self.args_edit_kind != crate::tui::app::ArgsEditKind::GlobalJava { return; }
        let parsed = super::instances_ctrl::parse_shell_like_args(self.args_edit_input.trim());
        let mut cfg = match LauncherConfig::load_s() {
            Ok(c) => c,
            Err(e) => { self.status_message = format!("❌ Failed to load global config: {}", e); return; }
        };
        cfg.extra_java_args = Some(parsed);
        match self.save_config_sync(&cfg) {
            Ok(_) => {
                self.is_editing_args = false;
                self.status_message = "✅ Saved global Java arguments".to_string();
            }
            Err(e) => { self.status_message = format!("❌ Failed to save global config: {}", e); }
        }
    }
}
