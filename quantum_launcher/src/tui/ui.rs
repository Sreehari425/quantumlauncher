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
            let status = if account.is_logged_in { " (logged in)" } else { "" };
            
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
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(area);

    // Status message
    let status = Paragraph::new(app.status_message.clone())
        .block(Block::default().borders(Borders::ALL).title(" Status "))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    // Help/keybinds
    let help = Paragraph::new("Press '?' for help | 'q' to quit")
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[1]);
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
            Line::from("• Offline         - Play without authentication"),
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
            if i == app.selected_account_type {
                item = item.style(Style::default().bg(Color::Yellow).fg(Color::Black));
            }
            item
        })
        .collect();

    let type_list = List::new(type_items)
        .block(Block::default().borders(Borders::ALL).title("Account Type"))
        .style(Style::default().fg(Color::White));
    f.render_widget(type_list, chunks[0]);

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

    // Password input (only for non-Microsoft accounts)
    if app.new_account_type != crate::tui::app::AccountType::Microsoft {
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
    
    if app.new_account_type == crate::tui::app::AccountType::Microsoft {
        instruction_lines.push("↑/↓: Select account type | Enter: Add account | Esc: Cancel".to_string());
        instruction_lines.push("Microsoft accounts will open OAuth2 flow".to_string());
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
