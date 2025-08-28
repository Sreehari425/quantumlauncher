// QuantumLauncher TUI Module

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;

pub use app::{App, AppResult};

/// Entry point for the TUI mode
pub fn run_tui() -> AppResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

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
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> AppResult<()> {
    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

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
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Left | KeyCode::Char('h') => app.previous_tab(),
                        KeyCode::Right if app.current_tab != app::TabId::Accounts => app.next_tab(),
                        KeyCode::Enter => app.select_item(),
                        KeyCode::Char('c') => app.set_tab(app::TabId::Create),
                        KeyCode::Char('i') => app.set_tab(app::TabId::Instances),
                        KeyCode::Char('s') => app.set_tab(app::TabId::Settings),
                        KeyCode::Char('a') => app.set_tab(app::TabId::Accounts),
                        KeyCode::Char('?') => app.toggle_help_popup(),
                        KeyCode::F(5) => app.refresh(),
                        KeyCode::Char('n') if app.current_tab == app::TabId::Create => {
                            app.is_editing_name = !app.is_editing_name;
                            if app.is_editing_name {
                                app.status_message = "Editing instance name. Press 'n' again to finish editing.".to_string();
                            } else {
                                app.status_message = "Finished editing instance name.".to_string();
                            }
                        }
                        // Account tab specific keys
                        KeyCode::Char('l') if app.current_tab == app::TabId::Accounts => {
                            if app.is_login_mode {
                                app.toggle_login_mode();
                                app.status_message = "Login cancelled.".to_string();
                            } else if let Some(account) = app.get_selected_account() {
                                if account.is_logged_in {
                                    app.logout_account();
                                    app.status_message = "Account logged out.".to_string();
                                } else {
                                    app.toggle_login_mode();
                                    app.status_message = "Login mode activated. Enter username and password.".to_string();
                                }
                            }
                        }
                        KeyCode::Char('n') if app.current_tab == app::TabId::Accounts => {
                            app.status_message = "Adding new account (feature not yet implemented).".to_string();
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

        if app.should_quit() {
            break;
        }
    }
    Ok(())
}
