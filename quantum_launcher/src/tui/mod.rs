// QuantumLauncher TUI Module

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tokio::sync::mpsc;

// Configuration constants
const TUI_REFRESH_INTERVAL_MS: u64 = 500; // Periodic refresh to override stdout/stderr interference

// Use the refactored app module
#[path = "app/mod.rs"]
mod app;
mod ui;

pub use app::{App, AppResult};

#[derive(Debug, Clone)]
pub enum AuthEvent {
    LoginStarted,
    LoginSuccess { account_data: ql_instances::auth::AccountData },
    LoginNeedsOtp,
    LoginError { error: String },
    LaunchStarted(String),
    LaunchSuccess(String, std::sync::Arc<std::sync::Mutex<tokio::process::Child>>),
    LaunchError(String, String),
    LaunchEnded(String),
    // New: per-instance log line streamed from the running game's stdout/stderr
    InstanceLogLine { instance_name: String, line: String },
    InstanceCreateStarted(String),
    InstanceCreateProgress { instance_name: String, message: String },
    InstanceCreateSuccess { instance_name: String },
    InstanceCreateError { instance_name: String, error: String },
    RefreshStarted,
    RefreshCompleted,
    RefreshData { instances: Vec<(String, String, String)> }, // (name, version, loader)
}

/// Entry point for the TUI mode
pub fn run_tui() -> AppResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Disable stdio logging so println!/eprintln! don't corrupt the TUI
    ql_core::print::set_stdio_logging_enabled(false);
    // Create async runtime and run the app
    let rt = tokio::runtime::Runtime::new()?;
    let app = App::new();
    let res = rt.block_on(run_app(&mut terminal, app));

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Re-enable stdio logging now that TUI is closed
    ql_core::print::set_stdio_logging_enabled(true);

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

/// Main event loop for the TUI
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> AppResult<()> {
    // Create authentication channel
    let (auth_tx, mut auth_rx) = mpsc::unbounded_channel::<AuthEvent>();
    app.set_auth_channel(auth_tx);

    // Track last refresh time for periodic refreshing
    let mut last_refresh = std::time::Instant::now();

    loop {
        // Sync TUI logs with core logger buffer (last 500 lines)
        {
            let latest = ql_core::print::get_logs_lines(Some(2000));
            if latest != app.game_logs {
                let was_at_bottom = app.logs_offset == 0 && app.logs_auto_follow;
                app.game_logs = latest;
                if was_at_bottom {
                    app.logs_offset = 0; // keep following
                }
            }
            // Clamp offset so it doesn't exceed available history
            let max_offset = app.game_logs.len().saturating_sub(app.logs_visible_lines);
            if app.logs_offset > max_offset { app.logs_offset = max_offset; }
        }

        // Check for auth events first
        if let Ok(auth_event) = auth_rx.try_recv() {
            app.handle_auth_event(auth_event);
        }

        // Check if we need a forced refresh (e.g., after launch events that might spam stdout)
        let needs_forced_refresh = app.check_and_reset_forced_refresh();
        
        // Check if it's time for periodic refresh
        let needs_periodic_refresh = last_refresh.elapsed().as_millis() >= TUI_REFRESH_INTERVAL_MS as u128;
        
        if needs_forced_refresh || needs_periodic_refresh {
            // Clear terminal and force a complete redraw to overwrite any stdout spam
            terminal.clear()?;
            if needs_periodic_refresh {
                last_refresh = std::time::Instant::now();
            }
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        // Handle keyboard input with timeout to allow auth events to be processed
        if let Ok(has_event) = crossterm::event::poll(std::time::Duration::from_millis(50)) {
            if has_event {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        // Handle delete confirmation popup
                        if app.show_delete_confirm {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    // Confirm deletion
                                    if let Some(idx) = app.instance_settings_instance {
                                        let name = app.instances[idx].name.clone();
                                        app.delete_instance(&name);
                                    }
                                    app.show_delete_confirm = false;
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    // Cancel deletion
                                    app.show_delete_confirm = false;
                                    app.status_message = "Instance deletion cancelled.".to_string();
                                }
                                _ => {}
                            }
                            continue;
                        }
                        // Handle help popup separately
                        if app.show_help_popup {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                                    app.toggle_help_popup();
                                }
                                _ => {}
                            }
                        } else {
                            // Normal key handling when help/help and delete popup are not shown
                            match key.code {
                        // Logs tab scrolling
                        KeyCode::Up if app.current_tab == app::TabId::Logs => {
                            app.logs_auto_follow = false;
                            app.logs_offset = app.logs_offset.saturating_add(1);
                        }
                        KeyCode::Down if app.current_tab == app::TabId::Logs => {
                            app.logs_offset = app.logs_offset.saturating_sub(1);
                            if app.logs_offset == 0 { app.logs_auto_follow = true; }
                        }
                        KeyCode::PageUp if app.current_tab == app::TabId::Logs => {
                            app.logs_auto_follow = false;
                            app.logs_offset = app.logs_offset.saturating_add(app.logs_visible_lines);
                        }
                        KeyCode::PageDown if app.current_tab == app::TabId::Logs => {
                            app.logs_offset = app.logs_offset.saturating_sub(app.logs_visible_lines);
                            if app.logs_offset == 0 { app.logs_auto_follow = true; }
                        }
                        KeyCode::Home if app.current_tab == app::TabId::Logs => {
                            app.logs_auto_follow = false;
                            // scroll to top
                            app.logs_offset = app.game_logs.len();
                        }
                        KeyCode::End if app.current_tab == app::TabId::Logs => {
                            // follow bottom
                            app.logs_offset = 0;
                            app.logs_auto_follow = true;
                        }
                        KeyCode::Char('c') if app.current_tab == app::TabId::Logs => {
                            app.clear_logs();
                            app.logs_offset = 0;
                            app.logs_auto_follow = true;
                        }
                        KeyCode::Char('j') if app.current_tab == app::TabId::Logs => {
                            // scroll down one line (towards bottom)
                            app.logs_offset = app.logs_offset.saturating_sub(1);
                            if app.logs_offset == 0 { app.logs_auto_follow = true; }
                        }
                        KeyCode::Char('k') if app.current_tab == app::TabId::Logs => {
                            // scroll up one line (away from bottom)
                            app.logs_auto_follow = false;
                            app.logs_offset = app.logs_offset.saturating_add(1);
                        }
                        KeyCode::Char('g') if app.current_tab == app::TabId::Logs => {
                            // go to top
                            app.logs_auto_follow = false;
                            app.logs_offset = app.game_logs.len();
                        }
                        KeyCode::Char('G') if app.current_tab == app::TabId::Logs => {
                            // go to bottom
                            app.logs_offset = 0;
                            app.logs_auto_follow = true;
                        }
                        // Instance-specific logs scrolling (Instance Settings -> Logs)
                        KeyCode::Up if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_auto_follow = false;
                            app.instance_logs_offset = app.instance_logs_offset.saturating_add(1);
                        }
                        KeyCode::Down if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_offset = app.instance_logs_offset.saturating_sub(1);
                            if app.instance_logs_offset == 0 { app.instance_logs_auto_follow = true; }
                        }
                        KeyCode::PageUp if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_auto_follow = false;
                            app.instance_logs_offset = app.instance_logs_offset.saturating_add(app.instance_logs_visible_lines);
                        }
                        KeyCode::PageDown if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_offset = app.instance_logs_offset.saturating_sub(app.instance_logs_visible_lines);
                            if app.instance_logs_offset == 0 { app.instance_logs_auto_follow = true; }
                        }
                        KeyCode::Home if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_auto_follow = false;
                            // scroll to very top (use current instance's buffer length in renderer; here we max out offset later)
                            // We'll clamp in renderer based on available lines
                            app.instance_logs_offset = app.instance_logs_offset.saturating_add(usize::MAX/2);
                        }
                        KeyCode::End if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs => {
                            app.instance_logs_offset = 0;
                            app.instance_logs_auto_follow = true;
                        }
                        // Account tab specific keys - must come first to override general patterns
                        KeyCode::Esc if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            app.toggle_add_account_mode();
                        }
                        KeyCode::Enter if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            app.add_new_account();
                        }
                        KeyCode::Up if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            if app.new_account_type == app::AccountType::ElyBy {
                                // For ElyBy accounts, don't change account type when in a field
                                app.prev_account_type();
                            } else {
                                app.prev_account_type();
                            }
                        }
                        KeyCode::Down if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            if app.new_account_type == app::AccountType::ElyBy {
                                // For ElyBy accounts, don't change account type when in a field
                                app.next_account_type();
                            } else {
                                app.next_account_type();
                            }
                        }
                        KeyCode::Backspace if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            app.remove_char_from_add_account_field();
                        }
                        KeyCode::Tab if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            // Switch between username/password/OTP fields for ElyBy accounts
                            if app.new_account_type == app::AccountType::ElyBy {
                                app.next_add_account_field();
                            }
                        }
                        KeyCode::Char('p') if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            // Toggle password visibility for ElyBy accounts
                            if app.new_account_type == app::AccountType::ElyBy {
                                app.toggle_password_visibility();
                            } else {
                                app.add_char_to_add_account_field('p');
                            }
                        }
                        KeyCode::Char(c) if app.current_tab == app::TabId::Accounts && app.is_add_account_mode => {
                            app.add_char_to_add_account_field(c);
                        }
                        // Create tab editing mode - handle escape to exit editing
                        KeyCode::Esc if app.current_tab == app::TabId::Create && app.is_editing_name => {
                            app.is_editing_name = false;
                            app.status_message = "Finished editing instance name.".to_string();
                        }
                        // General key handling
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if app.current_tab == app::TabId::Create && app.version_search_active {
                                // Exit version search instead of quitting
                                app.exit_version_search();
                            } else if app.current_tab == app::TabId::InstanceSettings {
                                // Esc in instance settings goes back to instances
                                app.current_tab = app::TabId::Instances;
                                app.status_message = "Returned to instances list".to_string();
                            } else if app.current_tab == app::TabId::Settings && app.about_selected == app::App::licenses_menu_index() && app.settings_focus == app::SettingsFocus::Middle {
                                // In Settings Licenses submenu: Esc returns focus to left pane
                                app.settings_focus = app::SettingsFocus::Left;
                            } else {
                                return Ok(());
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.current_tab == app::TabId::InstanceSettings {
                                app.navigate_instance_settings(-1);
                            } else if app.current_tab == app::TabId::Settings {
                                app.prev_item();
                            } else {
                                app.prev_item();
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.current_tab == app::TabId::InstanceSettings {
                                app.navigate_instance_settings(1);
                            } else if app.current_tab == app::TabId::Settings {
                                app.next_item();
                            } else {
                                app.next_item();
                            }
                        }
                        KeyCode::PageUp if app.current_tab == app::TabId::Settings => {
                            app.about_scroll = app.about_scroll.saturating_sub(8);
                        }
                        KeyCode::PageDown if app.current_tab == app::TabId::Settings => {
                            app.about_scroll = app.about_scroll.saturating_add(8);
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.current_tab == app::TabId::InstanceSettings {
                                app.prev_instance_settings_tab();
                            } else if app.current_tab == app::TabId::Settings {
                                // Settings: move focus back to left pane when on Licenses
                                if app.about_selected == app::App::licenses_menu_index() {
                                    app.settings_focus = app::SettingsFocus::Left;
                                } else {
                                    app.previous_tab();
                                }
                            } else {
                                app.previous_tab();
                            }
                        }
                        KeyCode::Right => {
                            if app.current_tab == app::TabId::InstanceSettings {
                                app.next_instance_settings_tab();
                            } else if app.current_tab == app::TabId::Settings {
                                // Settings: when on Licenses, move focus to middle pane
                                if app.about_selected == app::App::licenses_menu_index() {
                                    app.settings_focus = app::SettingsFocus::Middle;
                                } else {
                                    app.next_tab();
                                }
                            } else {
                                app.next_tab();
                            }
                        }
                        KeyCode::Enter if app.current_tab == app::TabId::Settings => {
                            // When hovering Licenses, Enter focuses the middle pane
                            if app.about_selected == app::App::licenses_menu_index() {
                                app.settings_focus = app::SettingsFocus::Middle;
                            }
                        }
                        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            // Shift+Enter: Launch the selected instance
                            app.launch_selected_instance();
                        }
                        KeyCode::Enter => {
                            if app.current_tab == app::TabId::InstanceSettings {
                                app.select_instance_settings_item();
                            } else if app.current_tab == app::TabId::Instances {
                                app.select_item(); // This will open instance settings
                            } else if app.current_tab == app::TabId::Create && !app.is_editing_name {
                                // If searching, create with filtered selection
                                if app.version_search_active {
                                    app.create_instance();
                                    continue;
                                }
                                // Handle Create tab instance creation
                                if app.new_instance_name.is_empty() {
                                    // If no name entered, start editing the name
                                    app.is_editing_name = true;
                                    app.status_message = "Editing instance name. Press Esc to finish editing.".to_string();
                                } else if !app.available_versions.is_empty() && !app.is_loading {
                                    // If name is entered and versions available, create instance
                                    app.create_instance();
                                } else if app.available_versions.is_empty() {
                                    app.status_message = "❌ No versions available. Press F5 to refresh.".to_string();
                                }
                            } else {
                                app.select_item();
                            }
                        }
                        // Create tab: live search typing
                        KeyCode::Backspace if app.current_tab == app::TabId::Create && !app.is_editing_name => {
                            if app.version_search_active {
                                app.remove_char_from_version_search();
                            }
                        }
                        // Only accept characters when search is active
                        KeyCode::Char(c) if app.current_tab == app::TabId::Create && !app.is_editing_name && app.version_search_active => {
                            app.add_char_to_version_search(c);
                        }
                        // Ctrl+S toggles search mode
                        KeyCode::Char('s') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if app.version_search_active {
                                app.exit_version_search();
                            } else {
                                app.start_version_search();
                            }
                        }
                        // Filter toggles in Create tab
                        KeyCode::Char('r') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.toggle_filter_release();
                        }
                        KeyCode::Char('b') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.toggle_filter_beta();
                        }
                        KeyCode::Char('a') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.toggle_filter_alpha();
                        }
                        // Ctrl+Shift+S doesn't exist; reuse Ctrl+Shift+S-like by Ctrl+S already used; use Ctrl+Shift+X alternative -> choose Ctrl+Shift+S? Not ideal.
                        // We'll use Ctrl+Shift+S emulated by Ctrl+P for Snapshot to avoid conflict.
                        KeyCode::Char('p') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.toggle_filter_snapshot();
                        }
                        // Reset filters: Ctrl+0
                        KeyCode::Char('0') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.reset_all_filters();
                        }
                        // Tab navigation
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.previous_tab(),
                        // Logs tab specific keys - must come before general 'c' key
                        KeyCode::Char('c') if app.current_tab == app::TabId::Logs => {
                            app.clear_logs();
                            app.status_message = "✅ Logs cleared".to_string();
                        }
                        KeyCode::Char('?') => app.toggle_help_popup(),
                        KeyCode::F(5) => {
                            // Trigger async refresh
                            app.start_refresh();
                        }
                        KeyCode::F(12) => {
                            // Force terminal clear and redraw (useful if debug output disrupted display)
                            terminal.clear()?;
                            app.status_message = "🔄 Terminal refreshed".to_string();
                        }
                        // Version type filters (Create tab)
                        KeyCode::F(6) if app.current_tab == app::TabId::Create => {
                            app.toggle_filter_release();
                            app.status_message = format!("Filter Release: {}", if app.filter_release {"on"} else {"off"});
                        }
                        KeyCode::F(7) if app.current_tab == app::TabId::Create => {
                            app.toggle_filter_snapshot();
                            app.status_message = format!("Filter Snapshot: {}", if app.filter_snapshot {"on"} else {"off"});
                        }
                        KeyCode::F(8) if app.current_tab == app::TabId::Create => {
                            app.toggle_filter_beta();
                            app.status_message = format!("Filter Beta: {}", if app.filter_beta {"on"} else {"off"});
                        }
                        KeyCode::F(9) if app.current_tab == app::TabId::Create => {
                            app.toggle_filter_alpha();
                            app.status_message = format!("Filter Alpha: {}", if app.filter_alpha {"on"} else {"off"});
                        }
                        KeyCode::F(10) if app.current_tab == app::TabId::Create => {
                            app.reset_all_filters();
                            app.status_message = "Filters reset".to_string();
                        }
                        KeyCode::Char('n') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.is_editing_name = !app.is_editing_name;
                            if app.is_editing_name {
                                app.status_message = "Editing instance name. Press Ctrl+N again to finish editing.".to_string();
                            } else {
                                app.status_message = "Finished editing instance name.".to_string();
                            }
                        }
                        KeyCode::Char('d') if app.current_tab == app::TabId::Create && key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.download_assets = !app.download_assets;
                            app.status_message = if app.download_assets {
                                "✅ Download assets enabled (sound/music will be available)".to_string()
                            } else {
                                "⚠️  Download assets disabled (faster creation, no sound/music)".to_string()
                            };
                        }
                        // Account tab specific keys
                        KeyCode::Char('l') if app.current_tab == app::TabId::Accounts && !app.is_add_account_mode => {
                            if let Some(account) = app.get_selected_account() {
                                if account.is_logged_in {
                                    app.logout_account();
                                    app.status_message = "Account logged out.".to_string();
                                } else {
                                    app.status_message = "Use 'n' to add a new account and login during creation.".to_string();
                                }
                            }
                        }
                        KeyCode::Char('n') if app.current_tab == app::TabId::Accounts => {
                            if !app.is_add_account_mode {
                                app.toggle_add_account_mode();
                                app.status_message = "Add account mode. Select type and enter credentials.".to_string();
                            }
                        }
                        KeyCode::Char('d') if app.current_tab == app::TabId::Accounts && !app.is_add_account_mode => {
                            app.set_default_account();
                        }
                        KeyCode::Backspace if app.current_tab == app::TabId::Create && app.is_editing_name => {
                            app.new_instance_name.pop();
                        }
                        KeyCode::Char(c) if app.current_tab == app::TabId::Create && app.is_editing_name => {
                            app.new_instance_name.push(c);
                        }
                        _ => {}
                    }
                        }
                    },
                    Event::Mouse(me) => {
                        match me.kind {
                            MouseEventKind::ScrollUp => {
                                if app.current_tab == app::TabId::Settings {
                                    app.about_scroll = app.about_scroll.saturating_sub(3);
                                }
                                if app.current_tab == app::TabId::Logs {
                                    app.logs_auto_follow = false;
                                    app.logs_offset = app.logs_offset.saturating_add(3);
                                }
                                if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs {
                                    app.instance_logs_auto_follow = false;
                                    app.instance_logs_offset = app.instance_logs_offset.saturating_add(3);
                                }
                            }
                            MouseEventKind::ScrollDown => {
                                if app.current_tab == app::TabId::Settings {
                                    app.about_scroll = app.about_scroll.saturating_add(3);
                                }
                                if app.current_tab == app::TabId::Logs {
                                    app.logs_offset = app.logs_offset.saturating_sub(3);
                                    if app.logs_offset == 0 { app.logs_auto_follow = true; }
                                }
                                if app.current_tab == app::TabId::InstanceSettings && app.instance_settings_tab == app::InstanceSettingsTab::Logs {
                                    app.instance_logs_offset = app.instance_logs_offset.saturating_sub(3);
                                    if app.instance_logs_offset == 0 { app.instance_logs_auto_follow = true; }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }
    Ok(())
}
