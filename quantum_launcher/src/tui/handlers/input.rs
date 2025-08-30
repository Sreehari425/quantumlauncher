// QuantumLauncher TUI - Input Event Handler

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::tui::app::{App, TabId, AccountType};

/// Handle keyboard input events
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    // Handle help popup separately
    if app.show_help_popup {
        handle_help_popup_input(app, key.code);
        return false;
    }

    // Handle context-specific input with more specific conditions first
    match key.code {
        // Account tab specific keys - must come first to override general patterns
        KeyCode::Esc if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            app.toggle_add_account_mode();
            false
        }
        KeyCode::Enter if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            app.add_new_account();
            false
        }
        KeyCode::Up if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            if app.new_account_type == AccountType::ElyBy {
                app.prev_account_type();
            } else {
                app.prev_account_type();
            }
            false
        }
        KeyCode::Down if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            if app.new_account_type == AccountType::ElyBy {
                app.next_account_type();
            } else {
                app.next_account_type();
            }
            false
        }
        KeyCode::Backspace if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            app.remove_char_from_add_account_field();
            false
        }
        KeyCode::Tab if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            // Switch between username/password/OTP fields for ElyBy accounts
            if app.new_account_type == AccountType::ElyBy {
                app.next_add_account_field();
            }
            false
        }
        KeyCode::Char('p') if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            // Toggle password visibility for ElyBy accounts
            if app.new_account_type == AccountType::ElyBy {
                app.toggle_password_visibility();
            } else {
                app.add_char_to_add_account_field('p');
            }
            false
        }
        KeyCode::Char(c) if app.current_tab == TabId::Accounts && app.is_add_account_mode => {
            app.add_char_to_add_account_field(c);
            false
        }
        // General key handling
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_item();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
            false
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.previous_tab();
            false
        }
        KeyCode::Right if app.current_tab != TabId::Accounts => {
            app.next_tab();
            false
        }
        KeyCode::Enter => {
            app.select_item();
            false
        }
        // Logs tab specific keys - must come before general 'c' key
        KeyCode::Char('c') if app.current_tab == TabId::Logs => {
            app.clear_logs();
            app.status_message = "✅ Logs cleared".to_string();
            false
        }
        KeyCode::Char('c') => {
            app.set_tab(TabId::Create);
            false
        }
        KeyCode::Char('i') => {
            app.set_tab(TabId::Instances);
            false
        }
        KeyCode::Char('s') => {
            app.set_tab(TabId::Settings);
            false
        }
        KeyCode::Char('a') => {
            app.set_tab(TabId::Accounts);
            false
        }
        KeyCode::Char('l') if app.current_tab != TabId::Accounts => {
            app.set_tab(TabId::Logs);
            false
        }
        KeyCode::Char('?') => {
            app.toggle_help_popup();
            false
        }
        KeyCode::F(5) => {
            app.refresh();
            false
        }
        KeyCode::Char('n') if app.current_tab == TabId::Create => {
            app.is_editing_name = !app.is_editing_name;
            if app.is_editing_name {
                app.status_message = "Editing instance name. Press 'n' again to finish editing.".to_string();
            } else {
                app.status_message = "Finished editing instance name.".to_string();
            }
            false
        }
        // Account tab specific keys
        KeyCode::Char('l') if app.current_tab == TabId::Accounts && !app.is_add_account_mode => {
            if let Some(account) = app.get_selected_account() {
                if account.is_logged_in {
                    app.logout_account();
                    app.status_message = "Account logged out.".to_string();
                } else {
                    app.status_message = "Use 'n' to add a new account and login during creation.".to_string();
                }
            }
            false
        }
        KeyCode::Char('n') if app.current_tab == TabId::Accounts => {
            if !app.is_add_account_mode {
                app.toggle_add_account_mode();
                app.status_message = "Add account mode. Select type and enter credentials.".to_string();
            }
            false
        }
        KeyCode::Char('d') if app.current_tab == TabId::Accounts && !app.is_add_account_mode => {
            app.set_default_account();
            false
        }
        KeyCode::Backspace if app.current_tab == TabId::Create && app.is_editing_name => {
            app.new_instance_name.pop();
            false
        }
        KeyCode::Char(c) if app.current_tab == TabId::Create && app.is_editing_name => {
            app.new_instance_name.push(c);
            false
        }
        _ => false
    }
}

/// Handle input when help popup is shown
fn handle_help_popup_input(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
            app.toggle_help_popup();
        }
        _ => {}
    }
}


