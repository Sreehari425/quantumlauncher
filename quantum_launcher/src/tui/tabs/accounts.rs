// QuantumLauncher TUI - Accounts Tab

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the accounts tab
pub fn render_accounts_tab(f: &mut Frame, area: Rect, app: &App) {
    if app.is_add_account_mode {
        render_add_account_form(f, area, app);
    } else {
        render_account_list(f, area, app);
    }
}

fn render_account_list(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),  // Account list
            Constraint::Length(8), // Login form or account actions
        ])
        .split(area);

    // Account List
    let account_items: Vec<ListItem> = app.accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let status = if account.account_type == "Offline" {
                " (offline - ready to launch)"
            } else if account.is_logged_in { 
                " (logged in)" 
            } else { 
                "" 
            };
            
            // Check if this account is the default
            let is_default = if let Some(ref current_account) = app.current_account {
                // Create the username with type modifier to match current_account format
                let username_modified = if account.account_type == "ElyBy" {
                    format!("{} (elyby)", account.username)
                } else if account.account_type == "LittleSkin" {
                    format!("{} (littleskin)", account.username)
                } else {
                    account.username.clone()
                };
                current_account == &username_modified
            } else {
                false
            };
            
            let default_indicator = if is_default { " ★ (default)" } else { "" };
            let content = format!("{} [{}]{}{}", account.username, account.account_type, status, default_indicator);
            
            let mut item = ListItem::new(content);
            if i == app.selected_account {
                item = item.style(Style::default().bg(Color::Yellow).fg(Color::Black));
            }
            item
        })
        .collect();

    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts"))
        .style(Style::default().fg(Color::White));

    f.render_widget(accounts_list, chunks[0]);

    // Bottom panel for login form or account actions
    // Account actions - no longer supporting separate login mode
    let selected_account_info = if let Some(account) = app.get_selected_account() {
        if account.is_logged_in {
            format!("Selected: {} (logged in)\nPress 'l' to logout, 'n' to add new account", account.username)
        } else {
            format!("Selected: {} (not logged in)\nPress 'n' to add new account (login during creation)", account.username)
        }
    } else {
        format!("Debug: {} accounts, selected index: {}\nPress 'n' to add new account", 
            app.accounts.len(), app.selected_account)
    };

    let account_info = Paragraph::new(selected_account_info)
        .block(Block::default().borders(Borders::ALL).title("Actions"))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(account_info, chunks[1]);
}

fn render_add_account_form(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Account type selection
            Constraint::Length(3), // Username input
            Constraint::Length(3), // Password input (conditional)
            Constraint::Length(3), // OTP input (conditional)
            Constraint::Min(1),    // Instructions/status
        ])
        .split(area);

    // Account type selection
    let account_types = crate::tui::app::AccountType::all();
    let type_items: Vec<ListItem> = account_types
        .iter()
        .enumerate()
        .map(|(i, account_type)| {
            let mut item = ListItem::new(Line::from(account_type.to_string()));
            if i == app.selected_account_type {
                item = item.style(Style::default().bg(Color::Yellow).fg(Color::Black));
            }
            item
        })
        .collect();
    
    // Create a ListState to control scrolling
    let mut list_state = ListState::default();
    if app.selected_account_type < account_types.len() {
        list_state.select(Some(app.selected_account_type));
    }

    let type_list = List::new(type_items)
        .block(Block::default().borders(Borders::ALL).title("Account Type"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));
    
    f.render_stateful_widget(type_list, chunks[0], &mut list_state);

    // Username input with focus indication for ElyBy
    let username_title = if app.new_account_type == crate::tui::app::AccountType::ElyBy && 
                           app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Username {
        "👤 Username/Email [CURRENTLY TYPING HERE]"
    } else {
        "Username/Email"
    };
    
    let username_style = if app.new_account_type == crate::tui::app::AccountType::ElyBy && 
                           app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Username {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };
    
    let username_input = Paragraph::new(app.new_account_username.as_str())
        .block(Block::default().borders(Borders::ALL).title(username_title))
        .style(username_style);
    f.render_widget(username_input, chunks[1]);

    // Password input (only for non-Offline accounts and ElyBy accounts)
    if app.new_account_type != crate::tui::app::AccountType::Offline {
        let password_title = if app.new_account_type == crate::tui::app::AccountType::ElyBy && 
                               app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Password {
            "Password [CURRENTLY TYPING HERE]"
        } else {
            "Password"
        };
        
        let password_style = if app.new_account_type == crate::tui::app::AccountType::ElyBy && 
                               app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Password {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        
        let password_display = if app.show_password {
            app.new_account_password.clone()
        } else {
            "*".repeat(app.new_account_password.len())
        };
        
        let password_input = Paragraph::new(password_display.as_str())
            .block(Block::default().borders(Borders::ALL).title(password_title))
            .style(password_style);
        f.render_widget(password_input, chunks[2]);
    } else if app.new_account_type == crate::tui::app::AccountType::Offline {
        let info_text = "Offline accounts use the specified username for playing";
        let info_paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Info"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(info_paragraph, chunks[2]);
    } else {
        let coming_soon = Paragraph::new("Coming soon...")
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(coming_soon, chunks[2]);
    }
    
    // OTP input (only if needed for ElyBy)
    if app.needs_otp {
        let otp_title = if app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Otp {
            "🔢 OTP Code [CURRENTLY TYPING HERE]"
        } else {
            "OTP Code (from authenticator app)"
        };
        
        let otp_style = if app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Otp {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        
        let otp_display = app.new_account_otp.as_ref().map(|s| s.as_str()).unwrap_or("");
        let otp_input = Paragraph::new(otp_display)
            .block(Block::default().borders(Borders::ALL).title(otp_title))
            .style(otp_style);
        f.render_widget(otp_input, chunks[3]);
    } else {
        let placeholder = Paragraph::new("")
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(placeholder, chunks[3]);
    }

    // Instructions and error messages
    let mut instruction_lines = vec![];
    
    // Microsoft option is removed
    if false {
        instruction_lines.push("Microsoft accounts will open OAuth2 flow".to_string());
    } else if app.new_account_type == crate::tui::app::AccountType::Offline {
        instruction_lines.push("Offline accounts use the specified username for playing".to_string());
        instruction_lines.push("Press Enter to add account".to_string());
    } else if app.new_account_type == crate::tui::app::AccountType::ElyBy {
        instruction_lines.push("Enter ElyBy username/email and password".to_string());
        instruction_lines.push("Use Tab to switch fields, Enter to authenticate".to_string());
        if app.needs_otp {
            instruction_lines.push("⚠️ Two-factor authentication required - enter OTP code".to_string());
        }
    } else {
        instruction_lines.push("Account type not yet implemented".to_string());
    }
    
    // Add error message if present
    if let Some(ref error) = app.login_error {
        instruction_lines.push(format!("❌ Error: {}", error));
    }
    
    let instructions_text = instruction_lines.join("\n");
    let instructions_paragraph = Paragraph::new(instructions_text)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions_paragraph, chunks[4]);
}
