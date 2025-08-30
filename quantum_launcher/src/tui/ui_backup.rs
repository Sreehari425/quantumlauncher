// QuantumLauncher TUI - UI Rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        block::Title,
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame,
};
use ql_core::LAUNCHER_VERSION_NAME;

use crate::tui::app::{App, TabId};

/// Main rendering function
pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer with status
        ])
        .split(f.area());

    render_header(f, chunks[0], app);
    render_main_content(f, chunks[1], app);
    render_footer(f, chunks[2], app);

    if app.is_loading {
        render_loading_popup(f);
    }

    if app.show_help_popup {
        render_help_popup(f, app);
    }
}

/// Render the header with tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let tabs = vec![
        Line::from("Instances (i)"),
        Line::from("Create (c)"),
        Line::from("Settings (s)"),
        Line::from("Accounts (a)"),
        Line::from("Logs (l)"),
    ];

    let selected_tab = match app.current_tab {
        TabId::Instances => 0,
        TabId::Create => 1,
        TabId::Settings => 2,
        TabId::Accounts => 3,
        TabId::Logs => 4,
    };

    let tabs = Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title(
            Title::from(format!(" QuantumLauncher TUI {} ", LAUNCHER_VERSION_NAME))
                .alignment(Alignment::Center),
        ))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bold())
        .select(selected_tab)
        .divider("│");

    f.render_widget(tabs, area);
}

/// Render the main content area based on current tab
fn render_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_tab {
        TabId::Instances => render_instances_tab(f, area, app),
        TabId::Create => render_create_tab(f, area, app),
        TabId::Settings => render_settings_tab(f, area, app),
        TabId::Accounts => render_accounts_tab(f, area, app),
        TabId::Logs => render_logs_tab(f, area, app),
    }
}

/// Render the instances tab
fn render_instances_tab(f: &mut Frame, area: Rect, app: &mut App) {
    if app.instances.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Instances ");
        let paragraph = Paragraph::new("No instances found.\nPress F5 to refresh or go to Create tab to make a new instance.")
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .instances
        .iter()
        .map(|instance| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&instance.name, Style::default().fg(Color::Yellow).bold()),
                    Span::raw(" "),
                ]),
                Line::from(vec![
                    Span::raw("  Version: "),
                    Span::styled(&instance.version, Style::default().fg(Color::Cyan)),
                    Span::raw(" | Loader: "),
                    Span::styled(&instance.loader, Style::default().fg(Color::Green)),
                ]),
            ])
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_instance));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Instances (↑/↓ to navigate, Enter to launch) "),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut list_state);
}

/// Render the create instance tab
fn render_create_tab(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instance name input
            Constraint::Min(0),    // Version list
        ])
        .margin(1)
        .split(area);

    // Instance name input
    let name_input = Paragraph::new(if app.new_instance_name.is_empty() {
        if app.is_editing_name {
            "Enter instance name... (editing)"
        } else {
            "Enter instance name... (press 'n' to edit)"
        }
    } else {
        &app.new_instance_name
    })
    .style(if app.new_instance_name.is_empty() {
        Style::default().fg(Color::Gray)
    } else if app.is_editing_name {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(if app.is_editing_name {
                " Instance Name (Editing - press 'n' to finish) "
            } else {
                " Instance Name (press 'n' to edit) "
            })
    );

    f.render_widget(name_input, chunks[0]);

    // Version list
    if app.available_versions.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Minecraft Versions ");
        let paragraph = Paragraph::new("Loading versions...\nPress F5 to refresh")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .available_versions
            .iter()
            .map(|version| {
                ListItem::new(Line::from(vec![
                    Span::styled(&version.name, Style::default().fg(Color::Cyan)),
                    if version.is_classic_server {
                        Span::styled(" (Classic Server)", Style::default().fg(Color::Yellow))
                    } else {
                        Span::raw("")
                    },
                ]))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_version));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Minecraft Versions (↑/↓ to select, Enter to create) "),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}

/// Render the settings tab
fn render_settings_tab(f: &mut Frame, area: Rect, _app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings ");
    let paragraph = Paragraph::new(vec![
        Line::from("Settings functionality coming soon!"),
        Line::from(""),
        Line::from("This will include:"),
        Line::from("• Java settings"),
        Line::from("• Memory allocation"),
        Line::from("• Theme preferences"),
        Line::from("• Launch options"),
    ])
    .block(block)
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render the accounts tab
fn render_accounts_tab(f: &mut Frame, area: Rect, app: &App) {
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

/// Render the footer with status and help
fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(35), Constraint::Length(20)])
        .split(area);

    // Status message
    let status = Paragraph::new(app.status_message.clone())
        .block(Block::default().borders(Borders::ALL).title(" Status "))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    // Default account info
    let account_info = if let Some(current_account) = &app.current_account {
        // Extract the base username without modifiers
        let display_name = if current_account.contains(" (elyby)") {
            current_account.replace(" (elyby)", "")
        } else if current_account.contains(" (littleskin)") {
            current_account.replace(" (littleskin)", "")
        } else {
            current_account.clone()
        };
        format!("Default: {}", display_name)
    } else {
        "No default account".to_string()
    };
    
    let account = Paragraph::new(account_info)
        .block(Block::default().borders(Borders::ALL).title(" Account "))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(account, chunks[1]);

    // Help/keybinds
    let help = Paragraph::new("'?' help | 'q' quit")
        .block(Block::default().borders(Borders::ALL).title(" Keys "))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

/// Render loading popup
fn render_loading_popup(f: &mut Frame) {
    let area = centered_rect(30, 7, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("Loading...")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let loading_text = Paragraph::new("Please wait...")
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(loading_text, area);

    // Simple loading indicator
    let progress_area = Rect {
        x: area.x + 2,
        y: area.y + 3,
        width: area.width - 4,
        height: 1,
    };

    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(0.5); // Static for now, could be animated

    f.render_widget(gauge, progress_area);
}

/// Helper function to center a rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render help popup with contextual controls based on current tab and state
fn render_help_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let help_text = get_contextual_help(app);

    let block = Block::default()
        .title("❓ Help & Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let help_paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    f.render_widget(help_paragraph, area);
}

/// Generate contextual help content based on current tab and state
fn get_contextual_help(app: &App) -> Vec<Line> {
    let mut help_text = vec![
        Line::from(vec![
            Span::styled("QuantumLauncher TUI Controls", Style::default().fg(Color::Yellow).bold())
        ]),
        Line::from(""),
    ];

    // Add tab-specific help based on current tab
    match app.current_tab {
        TabId::Instances => {
            help_text.extend(get_instances_help(app));
        }
        TabId::Create => {
            help_text.extend(get_create_help(app));
        }
        TabId::Settings => {
            help_text.extend(get_settings_help());
        }
        TabId::Accounts => {
            help_text.extend(get_accounts_help(app));
        }
        TabId::Logs => {
            help_text.extend(get_logs_help());
        }
    }

    // Add common/global help
    help_text.extend(get_global_help());

    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![
        Span::styled("Press '?' or Esc to close this help", Style::default().fg(Color::Green).italic())
    ]));

    help_text
}

/// Help for Instances tab
fn get_instances_help(_app: &App) -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("═══ INSTANCES TAB ═══", Style::default().fg(Color::Cyan).bold())
        ]),
        Line::from("↑/↓ or j/k         Navigate instance list"),
        Line::from("Enter              Launch selected instance"),
        Line::from("e                  Edit selected instance (coming soon)"),
        Line::from("d                  Delete selected instance (coming soon)"),
        Line::from("F5                 Refresh instance list"),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Select an instance and press Enter for launch info (use CLI for actual launching)")
        ]),
        Line::from(""),
    ]
}

/// Help for Create tab
fn get_create_help(app: &App) -> Vec<Line> {
    let mut help = vec![
        Line::from(vec![
            Span::styled("═══ CREATE INSTANCE TAB ═══", Style::default().fg(Color::Green).bold())
        ]),
    ];

    if app.is_editing_name {
        help.extend(vec![
            Line::from(vec![
                Span::styled("🎯 Currently editing instance name", Style::default().fg(Color::Yellow).italic())
            ]),
            Line::from("Type               Enter instance name"),
            Line::from("Backspace          Delete character"),
            Line::from("Enter              Finish editing name"),
            Line::from("Esc                Cancel editing"),
            Line::from(""),
        ]);
    } else {
        help.extend(vec![
            Line::from("↑/↓ or j/k         Navigate version list"),
            Line::from("n                  Edit instance name"),
            Line::from("Enter              Create instance"),
            Line::from(""),
        ]);
    }

    help.extend(vec![
        Line::from(vec![
            Span::styled("💡 Tips:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("• Choose a descriptive name for your instance"),
        Line::from("• Select the Minecraft version you want to play"),
        Line::from("• Press Enter when ready to create the instance"),
        Line::from(""),
    ]);

    help
}

/// Help for Settings tab
fn get_settings_help() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══ SETTINGS TAB ═══", Style::default().fg(Color::Magenta).bold())
        ]),
        Line::from("↑/↓ or j/k         Navigate settings"),
        Line::from("Enter/Space        Toggle setting values"),
        Line::from("←/→ or h/l         Adjust numeric values"),
        Line::from("r                  Reset to defaults"),
        Line::from("s                  Save settings"),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Configure launcher behavior and performance here")
        ]),
        Line::from(""),
    ]
}

/// Help for Accounts tab
fn get_accounts_help(app: &App) -> Vec<Line<'static>> {
    let mut help = vec![
        Line::from(vec![
            Span::styled("═══ ACCOUNTS TAB ═══", Style::default().fg(Color::Blue).bold())
        ]),
    ];

    if app.is_add_account_mode {
        help.extend(vec![
            Line::from(vec![
                Span::styled("🎯 Currently adding new account", Style::default().fg(Color::Yellow).italic())
            ]),
            Line::from("↑/↓ or j/k         Select account type"),
            Line::from("Tab                Switch between fields"),
            Line::from("Type               Enter credentials"),
            Line::from("Enter              Create account"),
            Line::from("Esc                Cancel adding account"),
            Line::from(""),
            Line::from("Account Types:"),
            Line::from("• Microsoft        - Official Minecraft account"),
            Line::from("• ElyBy           - Alternative auth service"),
            Line::from("• LittleSkin      - Alternative auth service"),
            Line::from("• Offline         - Play without authentication (specify username)"),
            Line::from(""),
        ]);
    } else {
        help.extend(vec![
            Line::from("↑/↓ or j/k         Navigate account list"),
            Line::from("l                  Logout selected account (if logged in)"),
            Line::from("n                  Add new account (with login for ElyBy/LittleSkin)"),
            Line::from("d                  Set selected account as default"),
            Line::from("r                  Refresh account status"),
            Line::from(""),
        ]);
    }

    help.extend(vec![
        Line::from(vec![
            Span::styled("💡 Account Status:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("🟢 Green          - Currently logged in"),
        Line::from("🔴 Red            - Not logged in"),
        Line::from("🟡 Yellow         - Login in progress"),
        Line::from("★                 - Default account (used for launching)"),
        Line::from("🔵 Blue           - Offline account (ready for launching)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("🌐 ElyBy Support:", Style::default().fg(Color::Cyan)),
        ]),
        Line::from("• Full username/password authentication"),
        Line::from("• Two-factor authentication (OTP) support"),
        Line::from("• Password visibility toggle"),
        Line::from("• Real-time error feedback"),
        Line::from(""),
    ]);

    help
}

/// Global help that applies to all tabs
fn get_global_help() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══ GLOBAL CONTROLS ═══", Style::default().fg(Color::White).bold())
        ]),
        Line::from("←/→ or h/l         Switch between tabs"),
        Line::from("i                  Go to Instances tab"),
        Line::from("c                  Go to Create tab"),
        Line::from("s                  Go to Settings tab"),
        Line::from("a                  Go to Accounts tab"),
        Line::from("?                  Show/hide this help popup"),
        Line::from("q                  Quit application"),
        Line::from("F5                 Refresh current data"),
        Line::from("F12                Force terminal refresh (if display corrupted)"),
        Line::from("Esc                Go back / Cancel current action"),
        Line::from(""),
    ]
}

fn render_add_account_form(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Account type selection
            Constraint::Length(3), // Username input
            Constraint::Length(3), // Password input or info
            Constraint::Length(3), // OTP input (if needed) or empty
            Constraint::Min(1),    // Instructions/status
        ])
        .split(area);

    // Account type selection
    let account_types = crate::tui::app::AccountType::all();
    let type_items: Vec<ListItem> = account_types
        .iter()
        .enumerate()
        .map(|(i, account_type)| {
            let content = account_type.to_string();
            let mut item = ListItem::new(content);
            // Ensure selected_account_type is within bounds
            if i == app.selected_account_type && app.selected_account_type < account_types.len() {
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
            "🔑 Password [CURRENTLY TYPING HERE]"
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
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(info_paragraph, chunks[2]);
    } else {
        let info_text = "Microsoft accounts use OAuth2 authentication";
        let info_paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Info"))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(info_paragraph, chunks[2]);
    }
    
    // OTP input (only if needed for ElyBy)
    if app.needs_otp {
        let otp_title = if app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Otp {
            "🔐 OTP Code [CURRENTLY TYPING HERE]"
        } else {
            "🔐 OTP Code"
        };
        
        let otp_style = if app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Otp {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        
        let empty_string = String::new();
        let otp_display = app.new_account_otp.as_ref().unwrap_or(&empty_string);
        let otp_input = Paragraph::new(otp_display.as_str())
            .block(Block::default().borders(Borders::ALL).title(otp_title))
            .style(otp_style);
        f.render_widget(otp_input, chunks[3]);
    } else {
        // Empty placeholder
        let empty_paragraph = Paragraph::new("")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(empty_paragraph, chunks[3]);
    }

    // Instructions and error messages
    let mut instruction_lines = vec![];
    
    // Microsoft option is removed
    if false {
        // This block is now unreachable but kept for future implementation
        instruction_lines.push("↑/↓: Select account type | Enter: Add account | Esc: Cancel".to_string());
        instruction_lines.push("Microsoft accounts will open OAuth2 flow".to_string());
    } else if app.new_account_type == crate::tui::app::AccountType::Offline {
        instruction_lines.push("↑/↓: Select account type | Enter: Add offline account | Esc: Cancel".to_string());
        instruction_lines.push("Offline accounts use the specified username for playing".to_string());
    } else if app.new_account_type == crate::tui::app::AccountType::ElyBy {
        instruction_lines.push("↑/↓: Select account type | Tab: Next field | p: Toggle password visibility".to_string());
        instruction_lines.push("Enter: Create account | Esc: Cancel".to_string());
        
        if app.needs_otp {
            instruction_lines.push("📱 Two-factor authentication required - enter OTP code".to_string());
        }
    } else {
        instruction_lines.push("↑/↓: Select account type | Tab: Next field | Enter: Add account | Esc: Cancel".to_string());
    }
    
    // Add error message if present
    if let Some(ref error) = app.login_error {
        instruction_lines.push("".to_string());
        instruction_lines.push(format!("❌ Error: {}", error));
    }
    
    let instructions_text = instruction_lines.join("\n");
    let instructions_paragraph = Paragraph::new(instructions_text)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions_paragraph, chunks[4]);
}

/// Render the logs tab
fn render_logs_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Game Logs ");
    
    let log_text = if app.game_logs.is_empty() {
        vec![
            Line::from("No game logs to display."),
            Line::from(""),
            Line::from("Launch a Minecraft instance to see logs here."),
            Line::from(""),
            Line::from("Recent activity:"),
            Line::from(app.status_message.clone()),
        ]
    } else {
        let mut lines = vec![
            Line::from(format!("Game Logs ({} lines):", app.game_logs.len())),
            Line::from(""),
        ];
        
        // Show last 20 lines of logs
        let start_idx = if app.game_logs.len() > 20 {
            app.game_logs.len() - 20
        } else {
            0
        };
        
        for log_line in &app.game_logs[start_idx..] {
            lines.push(Line::from(log_line.clone()));
        }
        
        lines
    };
    
    let paragraph = Paragraph::new(log_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Help for Logs tab
fn get_logs_help() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══ LOGS TAB ═══", Style::default().fg(Color::Magenta).bold())
        ]),
        Line::from("↑/↓ or j/k         Scroll through logs"),
        Line::from("c                  Clear logs"),
        Line::from("f                  Filter logs"),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Game output and launcher events are shown here")
        ]),
        Line::from(""),
    ]
}
    // If forced refresh is needed, clear the terminal first
    if app.check_and_reset_forced_refresh() {
        frame.render_widget(Clear, frame.size());
    }

    // Create main layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status
        ])
        .split(frame.size());

    // Render tabs
    let tabs = Tabs::new(vec!["Instances", "Create", "Settings", "Accounts", "Logs"])
        .block(
            Block::default()
                .title("QuantumLauncher TUI")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(match app.current_tab {
            TabId::Instances => 0,
            TabId::Create => 1,
            TabId::Settings => 2,
            TabId::Accounts => 3,
            TabId::Logs => 4,
        });

    frame.render_widget(tabs, main_chunks[0]);

    // Render content based on current tab
    match app.current_tab {
        TabId::Instances => render_instances_tab(frame, app, main_chunks[1]),
        TabId::Create => render_create_tab(frame, app, main_chunks[1]),
        TabId::Settings => render_settings_tab(frame, app, main_chunks[1]),
        TabId::Accounts => render_accounts_tab(frame, app, main_chunks[1]),
        TabId::Logs => render_logs_tab(frame, app, main_chunks[1]),
    }

    // Render status bar
    let status_paragraph = Paragraph::new(app.status_message.as_str())
        .block(
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(status_paragraph, main_chunks[2]);

    // Render help popup if active
    if app.show_help_popup {
        render_help_popup(frame);
    }

    // Render add account popup if active
    if app.is_add_account_mode {
        render_add_account_popup(frame, app);
    }

    // Render loading overlay if loading
    if app.is_loading {
        render_loading_overlay(frame);
    }
}

fn render_instances_tab<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Instances list
    let items: Vec<ListItem> = app
        .instances
        .iter()
        .enumerate()
        .map(|(i, instance)| {
            let style = if i == app.selected_instance {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&instance.name, style),
                Span::raw(" - "),
                Span::styled(&instance.version, Style::default().fg(Color::Gray)),
                Span::raw(" ("),
                Span::styled(&instance.loader, Style::default().fg(Color::Blue)),
                Span::raw(")"),
            ]))
        })
        .collect();

    let instances_list = List::new(items)
        .block(
            Block::default()
                .title("Minecraft Instances")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !app.instances.is_empty() {
        list_state.select(Some(app.selected_instance));
    }

    frame.render_stateful_widget(instances_list, chunks[0], &mut list_state);

    // Instance details
    let details_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(chunks[1]);

    // Instance info
    let mut info_lines = vec![];

    if let Some(instance) = app.instances.get(app.selected_instance) {
        info_lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&instance.name),
        ]));
        info_lines.push(Line::from(vec![
            Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&instance.version),
        ]));
        info_lines.push(Line::from(vec![
            Span::styled("Loader: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&instance.loader),
        ]));
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to launch", Style::default().fg(Color::Gray)),
        ]));
    } else {
        info_lines.push(Line::from(vec![
            Span::styled("No instances found.", Style::default().fg(Color::Yellow)),
        ]));
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("Use the ", Style::default().fg(Color::Gray)),
            Span::styled("Create", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" tab to create one.", Style::default().fg(Color::Gray)),
        ]));
    }

    let instance_info = Paragraph::new(info_lines)
        .block(
            Block::default()
                .title("Instance Details")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    frame.render_widget(instance_info, details_chunks[0]);

    // Account info
    let account_info = if let Some(ref current_account) = app.current_account {
        format!("Default Account: {}", current_account)
    } else if app.can_launch_offline() {
        "No default account (will use offline mode)".to_string()
    } else {
        "⚠️ No accounts configured".to_string()
    };

    let account_paragraph = Paragraph::new(account_info)
        .block(
            Block::default()
                .title("Account Status")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(if app.current_account.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        });

    frame.render_widget(account_paragraph, details_chunks[1]);
}

fn render_create_tab<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Instance name input
    let name_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[0]);

    let name_input = Paragraph::new(app.new_instance_name.as_str())
        .block(
            Block::default()
                .title("New Instance Name")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(if app.is_editing_name {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    frame.render_widget(name_input, name_chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("e", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to edit instance name", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to create instance", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(Color::Gray)),
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to select version", Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(
        Block::default()
            .title("Instructions")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(instructions, name_chunks[1]);

    // Version list
    let version_items: Vec<ListItem> = app
        .available_versions
        .iter()
        .enumerate()
        .map(|(i, version)| {
            let style = if i == app.selected_version {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let version_type = if version.name.contains("snapshot") {
                "Snapshot"
            } else if version.name.contains("pre") || version.name.contains("rc") {
                "Pre-release"
            } else {
                "Release"
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&version.name, style),
                Span::raw(" ("),
                Span::styled(version_type, Style::default().fg(Color::Blue)),
                Span::raw(")"),
            ]))
        })
        .collect();

    let versions_list = List::new(version_items)
        .block(
            Block::default()
                .title("Available Versions")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !app.available_versions.is_empty() {
        list_state.select(Some(app.selected_version));
    }

    frame.render_stateful_widget(versions_list, chunks[1], &mut list_state);
}

fn render_settings_tab<B: Backend>(frame: &mut Frame<B>, _app: &App, area: Rect) {
    let settings_paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Settings", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("This tab will contain settings in the future."),
        Line::from(""),
        Line::from("Planned settings:"),
        Line::from("• Java installation path"),
        Line::from("• Memory allocation"),
        Line::from("• Download directory"),
        Line::from("• Theme selection"),
        Line::from(""),
        Line::from(vec![
            Span::styled("For now, use the ", Style::default().fg(Color::Gray)),
            Span::styled("Accounts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" tab to manage accounts.", Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(
        Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(settings_paragraph, area);
}

fn render_accounts_tab<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Accounts list
    let account_items: Vec<ListItem> = app
        .accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let style = if i == app.selected_account {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let is_default = app.current_account.as_ref() == Some(&account.username);
            let default_marker = if is_default { " [DEFAULT]" } else { "" };
            
            let status_color = if account.is_logged_in {
                Color::Green
            } else {
                Color::Red
            };
            
            let status = if account.is_logged_in { "✓" } else { "✗" };

            // Clean up the display username by removing type suffixes
            let display_username = match account.account_type.as_str() {
                "ElyBy" => account.username.strip_suffix(" (elyby)").unwrap_or(&account.username),
                "LittleSkin" => account.username.strip_suffix(" (littleskin)").unwrap_or(&account.username),
                _ => &account.username,
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(display_username, style),
                Span::raw(" ("),
                Span::styled(&account.account_type, Style::default().fg(Color::Blue)),
                Span::raw(")"),
                Span::styled(default_marker, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
        })
        .collect();

    let accounts_list = List::new(account_items)
        .block(
            Block::default()
                .title("Accounts")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !app.accounts.is_empty() {
        list_state.select(Some(app.selected_account));
    }

    frame.render_stateful_widget(accounts_list, chunks[0], &mut list_state);

    // Account actions and info
    let actions_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(12)])
        .split(chunks[1]);

    // Account details
    let mut info_lines = vec![];

    if let Some(account) = app.accounts.get(app.selected_account) {
        let display_username = match account.account_type.as_str() {
            "ElyBy" => account.username.strip_suffix(" (elyby)").unwrap_or(&account.username),
            "LittleSkin" => account.username.strip_suffix(" (littleskin)").unwrap_or(&account.username),
            _ => &account.username,
        };
        
        info_lines.push(Line::from(vec![
            Span::styled("Username: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(display_username),
        ]));
        info_lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&account.account_type),
        ]));
        info_lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if account.is_logged_in { "Logged In" } else { "Logged Out" },
                Style::default().fg(if account.is_logged_in { Color::Green } else { Color::Red }),
            ),
        ]));
        info_lines.push(Line::from(vec![
            Span::styled("UUID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&account.uuid[..8]), // Show first 8 chars of UUID
            Span::raw("..."),
        ]));
        
        let is_default = app.current_account.as_ref() == Some(&account.username);
        if is_default {
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(vec![
                Span::styled("✓ Default Account", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        }
    } else {
        info_lines.push(Line::from(vec![
            Span::styled("No accounts configured", Style::default().fg(Color::Yellow)),
        ]));
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("n", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to add an account", Style::default().fg(Color::Gray)),
        ]));
    }

    let account_info = Paragraph::new(info_lines)
        .block(
            Block::default()
                .title("Account Details")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    frame.render_widget(account_info, actions_chunks[0]);

    // Action buttons
    let mut action_lines = vec![
        Line::from(vec![
            Span::styled("Available Actions:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    if !app.accounts.is_empty() {
        action_lines.push(Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Set as Default", Style::default().fg(Color::Gray)),
        ]));
        // action_lines.push(Line::from(vec![
        //     Span::styled("l", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        //     Span::styled(" - Login/Logout", Style::default().fg(Color::Gray)),
        // ]));
        // action_lines.push(Line::from(vec![
        //     Span::styled("x", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        //     Span::styled(" - Delete Account", Style::default().fg(Color::Gray)),
        // ]));
    }

    action_lines.push(Line::from(vec![
        Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" - Add New Account", Style::default().fg(Color::Gray)),
    ]));

    let actions = Paragraph::new(action_lines)
        .block(
            Block::default()
                .title("Actions")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    frame.render_widget(actions, actions_chunks[1]);
}

fn render_logs_tab<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    // Game logs
    let log_items: Vec<ListItem> = app
        .game_logs
        .iter()
        .map(|log| ListItem::new(Line::from(Span::raw(log))))
        .collect();

    let logs_list = List::new(log_items)
        .block(
            Block::default()
                .title("Game Logs")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(logs_list, chunks[0]);

    // Log controls
    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Clear logs", Style::default().fg(Color::Gray)),
        ]),
        Line::from("Logs will appear here when launching instances."),
    ])
    .block(
        Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(controls, chunks[1]);
}

fn render_help_popup<B: Backend>(frame: &mut Frame<B>) {
    let area = centered_rect(80, 80, frame.size());
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("QuantumLauncher TUI - Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Global Controls:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Tab/→", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Next tab  "),
            Span::styled("Shift+Tab/←", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Previous tab"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Navigate lists  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Select/Execute"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit  "),
            Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Toggle this help  "),
            Span::styled("r", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" - Refresh"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Instances Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Launch selected instance"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Create Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Edit instance name  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Create instance"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Accounts Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" - Add new account  "),
            Span::styled("d", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Set as default"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Logs Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Clear logs"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close this help", Style::default().fg(Color::Gray)),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(help_paragraph, area);
}

fn render_add_account_popup<B: Backend>(frame: &mut Frame<B>, app: &App) {
    let area = centered_rect(60, 40, frame.size());
    
    let mut text_lines = vec![
        Line::from(vec![
            Span::styled("Add New Account", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    // Account type selection
    text_lines.push(Line::from(vec![
        Span::styled("Account Type: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(&app.new_account_type.to_string(), Style::default().fg(Color::Yellow)),
    ]));
    text_lines.push(Line::from(vec![
        Span::styled("Use ", Style::default().fg(Color::Gray)),
        Span::styled("←/→", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" to change account type", Style::default().fg(Color::Gray)),
    ]));
    text_lines.push(Line::from(""));

    // Username field
    let username_style = if app.new_account_type != crate::tui::app::AccountType::ElyBy 
        || app.add_account_field_focus == AddAccountFieldFocus::Username {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
    };
    
    text_lines.push(Line::from(vec![
        Span::styled("Username: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(&app.new_account_username, username_style),
    ]));

    // Password field (only for ElyBy accounts)
    if app.new_account_type == crate::tui::app::AccountType::ElyBy {
        let password_display = if app.show_password {
            &app.new_account_password
        } else {
            &"*".repeat(app.new_account_password.len())
        };
        
        let password_style = if app.add_account_field_focus == AddAccountFieldFocus::Password {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default()
        };
        
        text_lines.push(Line::from(vec![
            Span::styled("Password: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(password_display, password_style),
        ]));

        // OTP field (if needed)
        if app.needs_otp {
            let otp_display = app.new_account_otp.as_ref().unwrap_or(&String::new());
            let otp_style = if app.add_account_field_focus == AddAccountFieldFocus::Otp {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default()
            };
            
            text_lines.push(Line::from(vec![
                Span::styled("OTP Code: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(otp_display, otp_style),
            ]));
        }
    }

    text_lines.push(Line::from(""));

    // Error message
    if let Some(ref error) = app.login_error {
        text_lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(error, Style::default().fg(Color::Red)),
        ]));
        text_lines.push(Line::from(""));
    }

    // Instructions
    text_lines.push(Line::from(vec![
        Span::styled("Controls:", Style::default().add_modifier(Modifier::BOLD)),
    ]));
    
    if app.new_account_type == crate::tui::app::AccountType::ElyBy {
        text_lines.push(Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Switch field  ", Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+P", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Toggle password visibility", Style::default().fg(Color::Gray)),
        ]));
    }
    
    text_lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" - Add account  ", Style::default().fg(Color::Gray)),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled(" - Cancel", Style::default().fg(Color::Gray)),
    ]));

    let add_account_paragraph = Paragraph::new(text_lines)
        .block(
            Block::default()
                .title("Add Account")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(add_account_paragraph, area);
}

fn render_loading_overlay<B: Backend>(frame: &mut Frame<B>) {
    let area = centered_rect(30, 10, frame.size());
    
    let loading_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Loading...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Please wait", Style::default().fg(Color::Gray)),
        ]),
    ];

    let loading_paragraph = Paragraph::new(loading_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(Clear, area);
    frame.render_widget(loading_paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
