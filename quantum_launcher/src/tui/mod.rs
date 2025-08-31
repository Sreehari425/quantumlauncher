// QuantumLauncher TUI Module

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tokio::sync::mpsc;

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
}

/// Entry point for the TUI mode
pub fn run_tui() -> AppResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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

    loop {
        // Check for auth events first
        if let Ok(auth_event) = auth_rx.try_recv() {
            app.handle_auth_event(auth_event);
        }

        // Check if we need a forced refresh (e.g., after launch events that might spam stdout)
        if app.check_and_reset_forced_refresh() {
            // Clear terminal and force a complete redraw to overwrite any stdout spam
            terminal.clear()?;
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        // Handle keyboard input with timeout to allow auth events to be processed
        if let Ok(has_event) = crossterm::event::poll(std::time::Duration::from_millis(50)) {
            if has_event {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                // Handle help popup separately
                if app.show_help_popup {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                            app.toggle_help_popup();
                        }
                        _ => {}
                    }
                } else {
                    // Normal key handling when help popup is not shown
                    match key.code {
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
                        // General key handling
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Left | KeyCode::Char('h') => app.previous_tab(),
                        KeyCode::Right if app.current_tab != app::TabId::Accounts => app.next_tab(),
                        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            // Shift+Enter: Launch the selected instance
                            app.launch_selected_instance();
                        }
                        KeyCode::Enter if app.current_tab != app::TabId::Instances => app.select_item(),
                        // Logs tab specific keys - must come before general 'c' key
                        KeyCode::Char('c') if app.current_tab == app::TabId::Logs => {
                            app.clear_logs();
                            app.status_message = "✅ Logs cleared".to_string();
                        }
                        KeyCode::Char('c') => app.set_tab(app::TabId::Create),
                        KeyCode::Char('i') => app.set_tab(app::TabId::Instances),
                        KeyCode::Char('s') => app.set_tab(app::TabId::Settings),
                        KeyCode::Char('a') => app.set_tab(app::TabId::Accounts),
                        KeyCode::Char('l') if app.current_tab != app::TabId::Accounts => app.set_tab(app::TabId::Logs),
                        KeyCode::Char('?') => app.toggle_help_popup(),
                        KeyCode::F(5) => app.refresh(),
                        KeyCode::F(12) => {
                            // Force terminal clear and redraw (useful if debug output disrupted display)
                            terminal.clear()?;
                            app.status_message = "🔄 Terminal refreshed".to_string();
                        }
                        KeyCode::Char('n') if app.current_tab == app::TabId::Create => {
                            app.is_editing_name = !app.is_editing_name;
                            if app.is_editing_name {
                                app.status_message = "Editing instance name. Press 'n' again to finish editing.".to_string();
                            } else {
                                app.status_message = "Finished editing instance name.".to_string();
                            }
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
                    }
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }
    Ok(())
}
