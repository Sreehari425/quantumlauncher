// QuantumLauncher TUI - UI Rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame,
};

use crate::tui::app::{App, TabId};

/// Main rendering function
pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with tabs (consistent for all pages)
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

    if app.show_delete_confirm {
        render_delete_confirm_popup(f, app);
    }
}
/// Render confirmation popup for instance deletion
fn render_delete_confirm_popup(f: &mut Frame, _app: &App) {
    let popup_area = centered_rect(50, 30, f.area());
    
    let block = Block::default()
        .title(" Confirm Instance Deletion ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).bold());
    let lines = vec![
        Line::from(""),
        Line::from("Are you sure you want to permanently delete this instance?"),
        Line::from(""),
        Line::from("This action cannot be undone!"),
        Line::from(""),
        Line::from("Press 'Y' to confirm, 'N' or Esc to cancel."),
        Line::from(""),
    ];
    let para = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(Clear, popup_area); // Clear the popup area
    f.render_widget(para, popup_area);
}

/// Render the header with tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    // Normal main tabs (same for all pages including instance settings)
    let tabs = vec![
        Line::from("Instances"),
        Line::from("Create"),
        Line::from("Settings"),
        Line::from("Accounts"),
        Line::from("Logs"),
    ];

    let selected_tab = match app.current_tab {
        TabId::Instances => 0,
        TabId::Create => 1,
        TabId::Settings => 2,
        TabId::Accounts => 3,
        TabId::Logs => 4,
        TabId::InstanceSettings => 0, // Show as Instances tab when in instance settings
    };

    let tabs_widget = Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title(" QuantumLauncher "))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(selected_tab);
    f.render_widget(tabs_widget, area);
}

/// Render the main content area based on current tab
fn render_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_tab {
        TabId::Instances => render_instances_tab(f, area, app),
        TabId::Create => render_create_tab(f, area, app),
        TabId::Settings => render_settings_tab(f, area, app),
        TabId::Accounts => render_accounts_tab(f, area, app),
        TabId::Logs => render_logs_tab(f, area, app),
        TabId::InstanceSettings => render_instance_settings_tab(f, area, app),
    }
}

/// Render the instances tab
fn render_instances_tab(f: &mut Frame, area: Rect, app: &mut App) {
    if app.instances.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Instances ");
        let paragraph = Paragraph::new("No instances found.\nPress F5 to refresh or use Tab to navigate to Create tab to make a new instance.")
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
            let name_spans = vec![
                Span::styled(&instance.name, Style::default().fg(Color::Yellow).bold()),
                Span::raw(" "),
            ];
            
            ListItem::new(vec![
                Line::from(name_spans),
                Line::from(vec![
                    Span::raw("  Version: "),
                    Span::styled(&instance.version, Style::default().fg(Color::Cyan)),
                    Span::raw(" | Loader: "),
                    Span::styled(&instance.loader, Style::default().fg(Color::Green)),
                    Span::raw(" | Status: "),
                    Span::styled(
                        if instance.is_running { "running" } else { "stopped" },
                        Style::default().fg(if instance.is_running { Color::Red } else { Color::Gray })
                    ),
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
            Constraint::Length(3), // Download assets checkbox
            Constraint::Length(3), // Create button
            Constraint::Min(0),    // Version list
        ])
        .margin(1)
        .split(area);

    // Instance name input
    let name_input = Paragraph::new(if app.new_instance_name.is_empty() {
        if app.is_editing_name {
            "Enter instance name... (editing)"
        } else {
            "Enter instance name... (press Ctrl+N to edit)"
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
                " Instance Name (Editing - press Esc to finish) "
            } else {
                " Instance Name (press Ctrl+N to edit) "
            })
    );
    f.render_widget(name_input, chunks[0]);

    // Download assets checkbox
    let checkbox_text = if app.download_assets {
        "☑ Download assets? (Enables sound/music, but slower)"
    } else {
        "☐ Download assets? (Enables sound/music, but slower)"
    };
    let checkbox = Paragraph::new(checkbox_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Options (press Ctrl+D to toggle) ")
        );
    f.render_widget(checkbox, chunks[1]);

    // Create button and status
    let can_create = !app.new_instance_name.is_empty() && !app.available_versions.is_empty();
    let button_text = if app.is_loading {
        "Creating instance... Please wait"
    } else if app.new_instance_name.is_empty() {
        "Press Enter to name instance"
    } else if can_create {
        "Press Enter to create instance"
    } else {
        "No version selected"
    };
    
    let button_style = if app.is_loading {
        Style::default().fg(Color::Yellow)
    } else if can_create {
        Style::default().fg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Red)
    };
    
    let create_button = Paragraph::new(button_text)
        .style(button_style)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Create Instance ")
        );
    f.render_widget(create_button, chunks[2]);

    // Version list - enhanced with better styling
    if app.available_versions.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Minecraft Versions ")
            .title_style(Style::default().fg(Color::Cyan).bold());
        let paragraph = Paragraph::new(vec![
            Line::from("Loading versions..."),
            Line::from(""),
            Line::from("Press F5 to refresh"),
            Line::from(""),
            Line::from("Supported versions:"),
            Line::from("  • Release versions"),
            Line::from("  • Snapshot versions"),
            Line::from("  • Beta/Alpha versions"),
        ])
        .style(Style::default().fg(Color::Gray))
        .block(block)
        .alignment(Alignment::Center);
        f.render_widget(paragraph, chunks[3]);
    } else {
        let items: Vec<ListItem> = app
            .available_versions
            .iter()
            .enumerate()
            .map(|(i, version)| {
                let is_selected = i == app.selected_version;
                let (type_label, type_color) = if version.name.contains("w") || version.name.contains("-pre") || version.name.contains("-rc") {
                    ("Snapshot", Color::Yellow)
                } else if version.name.contains("a") || version.name.contains("b") {
                    ("Alpha/Beta", Color::Magenta)
                } else {
                    ("Release", Color::Green)
                };
                
                ListItem::new(vec![
                    Line::from(vec![
                        if is_selected { 
                            Span::styled("▶ ", Style::default().fg(Color::Yellow).bold())
                        } else { 
                            Span::raw("  ")
                        },
                        Span::styled(&version.name, Style::default().fg(Color::Cyan).bold()),
                        Span::raw(" "),
                        Span::styled(format!("[{}]", type_label), Style::default().fg(type_color)),
                        if version.is_classic_server {
                            Span::styled(" (Server)", Style::default().fg(Color::Blue))
                        } else {
                            Span::raw("")
                        },
                    ])
                ])
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_version));

        let selected_version_name = if let Some(version) = app.available_versions.get(app.selected_version) {
            format!(" Minecraft Versions - Selected: {} ", version.name)
        } else {
            " Minecraft Versions ".to_string()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(selected_version_name)
                    .title_style(Style::default().fg(Color::Cyan).bold())
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("");

        f.render_stateful_widget(list, chunks[3], &mut list_state);
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
                // The account.username already contains the full modified username (e.g., "ThinThyme397339 (elyby)")
                // so we can directly compare it with current_account
                current_account == &account.username
            } else {
                false
            };
            
            let default_indicator = if is_default { " * (default)" } else { "" };
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
        .title("Help & Controls")
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
        TabId::InstanceSettings => {
            help_text.extend(get_instance_settings_help(app));
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
        Line::from("Shift+Enter        Launch selected instance"),
        Line::from("e                  Edit selected instance (coming soon)"),
        Line::from("d                  Delete selected instance (coming soon)"),
        Line::from("F5                 Refresh instance list"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
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
                Span::styled("Currently editing instance name", Style::default().fg(Color::Yellow).italic())
            ]),
            Line::from("Type               Enter instance name"),
            Line::from("Backspace          Delete character"),
            Line::from("Esc                Finish editing name"),
            Line::from(""),
        ]);
    } else {
        help.extend(vec![
            Line::from("↑/↓ or j/k         Navigate version list"),
            Line::from("Ctrl+N             Edit name / Create instance"),
            Line::from("Ctrl+D             Toggle download assets"),
            Line::from(""),
        ]);
    }

    help.extend(vec![
        Line::from(vec![
            Span::styled("Create Instance Guide:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("1. Press Ctrl+N to edit instance name"),
        Line::from("2. Select Minecraft version (↑/↓)"), 
        Line::from("3. Toggle asset download (press Ctrl+D)"),
        Line::from("   ☑ Enabled: Slower download, with sound/music"),
        Line::from("   ☐ Disabled: Faster download, no sound/music"),
        Line::from("4. Press Enter to create"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Cyan)),
            Span::raw("Simple controls: Ctrl+N to edit/create, Ctrl+D to toggle assets")
        ]),
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
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
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
                Span::styled("Currently adding new account", Style::default().fg(Color::Yellow).italic())
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
            Span::styled("Account Status:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("Green          - Currently logged in"),
        Line::from("Red            - Not logged in"),
        Line::from("Yellow         - Login in progress"),
        Line::from("*              - Default account (used for launching)"),
        Line::from("Blue           - Offline account (ready for launching)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("ElyBy Support:", Style::default().fg(Color::Cyan)),
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
            Constraint::Length(10), // Account type selection - much bigger to show all 4 options (4 items + 2 borders + 4 padding)
            Constraint::Length(3), // Username input - normal size
            Constraint::Length(3), // Password input or info - normal size
            Constraint::Length(3), // OTP input (if needed) or empty - normal size
            Constraint::Min(3),    // Instructions/status - minimum space
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

    // Username input with focus indication for ElyBy and LittleSkin
    let username_title = if (app.new_account_type == crate::tui::app::AccountType::ElyBy || 
                            app.new_account_type == crate::tui::app::AccountType::LittleSkin) && 
                           app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Username {
        "Username/Email [CURRENTLY TYPING HERE]"
    } else {
        "Username/Email"
    };
    
    let username_style = if (app.new_account_type == crate::tui::app::AccountType::ElyBy || 
                            app.new_account_type == crate::tui::app::AccountType::LittleSkin) && 
                           app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Username {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };
    
    let username_input = Paragraph::new(app.new_account_username.as_str())
        .block(Block::default().borders(Borders::ALL).title(username_title))
        .style(username_style);
    f.render_widget(username_input, chunks[1]);

    // Password input (for ElyBy, LittleSkin, and Microsoft accounts)
    if app.new_account_type != crate::tui::app::AccountType::Offline {
        let password_title = if (app.new_account_type == crate::tui::app::AccountType::ElyBy || 
                                app.new_account_type == crate::tui::app::AccountType::LittleSkin) && 
                               app.add_account_field_focus == crate::tui::app::AddAccountFieldFocus::Password {
            "Password [CURRENTLY TYPING HERE]"
        } else {
            "Password"
        };
        
        let password_style = if (app.new_account_type == crate::tui::app::AccountType::ElyBy || 
                                app.new_account_type == crate::tui::app::AccountType::LittleSkin) && 
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
            "OTP Code [CURRENTLY TYPING HERE]"
        } else {
            "OTP Code"
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
    
    match app.new_account_type {
        crate::tui::app::AccountType::Microsoft => {
            instruction_lines.push("↑/↓: Select account type | Enter: Add account | Esc: Cancel".to_string());
            instruction_lines.push("Warning: Microsoft accounts are not implemented yet - coming soon!".to_string());
        }
        crate::tui::app::AccountType::Offline => {
            instruction_lines.push("↑/↓: Select account type | Enter: Add offline account | Esc: Cancel".to_string());
            instruction_lines.push("Offline accounts use the specified username for playing".to_string());
        }
        crate::tui::app::AccountType::ElyBy => {
            instruction_lines.push("↑/↓: Select account type | Tab: Next field | p: Toggle password visibility".to_string());
            instruction_lines.push("Enter: Create account | Esc: Cancel".to_string());
            
            if app.needs_otp {
                instruction_lines.push("Two-factor authentication required - enter OTP code".to_string());
            }
        }
        crate::tui::app::AccountType::LittleSkin => {
            instruction_lines.push("↑/↓: Select account type | Tab: Next field | Enter: Add account | Esc: Cancel".to_string());
            instruction_lines.push("Warning: LittleSkin accounts are not implemented yet - coming soon!".to_string());
        }
    }
    
    // Add error message if present
    if let Some(ref error) = app.login_error {
        instruction_lines.push("".to_string());
        instruction_lines.push(format!("Error: {}", error));
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
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Game output and launcher events are shown here")
        ]),
        Line::from(""),
    ]
}

/// Render the instance settings tab
fn render_instance_settings_tab(f: &mut Frame, area: Rect, app: &mut App) {
    if let Some(instance_idx) = app.instance_settings_instance {
        if let Some(instance) = app.instances.get(instance_idx) {
            // Create a clean, modern layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7), // Instance info card (expanded)
                    Constraint::Length(3), // Sub-tabs
                    Constraint::Min(0),    // Content
                ])
                .split(area);

            // Render instance info card with better styling
            render_instance_info_card(f, chunks[0], instance);

            // Render sub-tabs with better styling
            let sub_tabs = vec![
                Line::from("Overview"),
                Line::from("Mods"), 
                Line::from("Settings"),
                Line::from("Logs"),
            ];

            let selected_sub_tab = match app.instance_settings_tab {
                crate::tui::app::InstanceSettingsTab::Overview => 0,
                crate::tui::app::InstanceSettingsTab::Mod => 1,
                crate::tui::app::InstanceSettingsTab::Setting => 2,
                crate::tui::app::InstanceSettingsTab::Logs => 3,
            };

            let sub_tabs_widget = Tabs::new(sub_tabs)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Instance Management ")
                        .title_style(Style::default().fg(Color::Cyan).bold())
                )
                .highlight_style(Style::default().fg(Color::Yellow).bold().bg(Color::DarkGray))
                .select(selected_sub_tab);
            f.render_widget(sub_tabs_widget, chunks[1]);

            // Render content based on selected sub-tab
            match app.instance_settings_tab {
                crate::tui::app::InstanceSettingsTab::Overview => render_instance_overview(f, chunks[2], app.instance_settings_selected, instance),
                crate::tui::app::InstanceSettingsTab::Mod => render_instance_mods(f, chunks[2], &instance.name),
                crate::tui::app::InstanceSettingsTab::Setting => render_instance_settings(f, chunks[2], app.instance_settings_selected, instance),
                crate::tui::app::InstanceSettingsTab::Logs => render_instance_logs(f, chunks[2], &instance.name),
            }
        }
    }
}

/// Render a beautiful instance info card
fn render_instance_info_card(f: &mut Frame, area: Rect, instance: &crate::tui::app::Instance) {
    let status_color = if instance.is_running { Color::Green } else { Color::Gray };
    let status_text = if instance.is_running { "● RUNNING" } else { "○ STOPPED" };

    // Create a nice card layout
    let card_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Length(1), // Empty line
            Constraint::Length(1), // Version info
            Constraint::Length(1), // Loader info  
            Constraint::Length(1), // Status info
            Constraint::Length(1), // Navigation help
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Instance Details ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .border_style(Style::default().fg(Color::Blue));

    // Instance name (prominent)
    let name_text = Paragraph::new(
        Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Yellow)),
            Span::styled(&instance.name, Style::default().fg(Color::Cyan).bold()),
        ])
    ).alignment(Alignment::Left);
    
    // Version info with nice formatting
    let version_text = Paragraph::new(
        Line::from(vec![
            Span::raw("  Minecraft Version: "),
            Span::styled(&instance.version, Style::default().fg(Color::Green).bold()),
        ])
    ).alignment(Alignment::Left);
    
    // Loader info
    let loader_text = Paragraph::new(
        Line::from(vec![
            Span::raw("  Mod Loader: "),
            Span::styled(&instance.loader, Style::default().fg(Color::Yellow).bold()),
        ])
    ).alignment(Alignment::Left);
    
    // Status with colored indicator
    let status_text_widget = Paragraph::new(
        Line::from(vec![
            Span::raw("  Status: "),
            Span::styled(status_text, Style::default().fg(status_color).bold()),
        ])
    ).alignment(Alignment::Left);
    
    // Navigation help
    let help_text = Paragraph::new(
        Line::from(vec![
            Span::styled("  ← → ", Style::default().fg(Color::Gray)),
            Span::raw("Switch tabs  "),
            Span::styled("↑ ↓ ", Style::default().fg(Color::Gray)),
            Span::raw("Navigate  "),
            Span::styled("Enter ", Style::default().fg(Color::Gray)),
            Span::raw("Select  "),
            Span::styled("Esc ", Style::default().fg(Color::Gray)),
            Span::raw("Back"),
        ])
    ).alignment(Alignment::Left);

    // Render the card background
    f.render_widget(block, area);
    
    // Render content inside the card
    f.render_widget(name_text, card_chunks[0]);
    f.render_widget(version_text, card_chunks[2]);
    f.render_widget(loader_text, card_chunks[3]);
    f.render_widget(status_text_widget, card_chunks[4]);
    f.render_widget(help_text, card_chunks[5]);
}

/// Render instance overview tab
fn render_instance_overview(f: &mut Frame, area: Rect, selected_index: usize, instance: &crate::tui::app::Instance) {
    // Create a two-column layout for better organization
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Actions column
            Constraint::Percentage(60), // Info column
        ])
        .split(area);

    // Left side: Quick Actions
    render_instance_actions(f, main_chunks[0], selected_index, instance);
    
    // Right side: Instance Statistics and Info
    render_instance_info_panel(f, main_chunks[1]);
}

/// Render the quick actions panel
fn render_instance_actions(f: &mut Frame, area: Rect, selected_index: usize, instance: &crate::tui::app::Instance) {
    let actions_block = Block::default()
        .borders(Borders::ALL)
        .title(" Quick Actions ")
        .title_style(Style::default().fg(Color::Green).bold())
        .border_style(Style::default().fg(Color::Green));

    let actions_items = vec![
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  ▶ ", if instance.is_running {
                    Style::default().fg(Color::DarkGray).bold()
                } else {
                    Style::default().fg(Color::Green).bold()
                }),
                Span::styled("Launch Instance", if instance.is_running {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White).bold()
                }),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    if instance.is_running {
                        "Instance is already running"
                    } else {
                        "Start playing this instance"
                    },
                    Style::default().fg(Color::Gray)
                ),
            ]),
        ]).style(if selected_index == 0 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
        
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  ⏹ ", if instance.is_running { 
                    Style::default().fg(Color::Red).bold() 
                } else { 
                    Style::default().fg(Color::DarkGray).bold() 
                }),
                Span::styled("Force Stop", if instance.is_running {
                    Style::default().fg(Color::White).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    if instance.is_running {
                        "Kill running instance process"
                    } else {
                        "Instance not running"
                    }, 
                    Style::default().fg(Color::Gray)
                ),
            ]),
        ]).style(if selected_index == 1 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
        
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  Folder ", Style::default().fg(Color::Blue).bold()),
                Span::styled("Open Folder", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("Browse instance directory", Style::default().fg(Color::Gray)),
            ]),
        ]).style(if selected_index == 2 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
    ];

    let actions_list = List::new(actions_items)
        .block(actions_block)
        .highlight_symbol("  ");
    f.render_widget(actions_list, area);
}

/// Render the info panel with statistics
fn render_instance_info_panel(f: &mut Frame, area: Rect) {
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Instance stats
            Constraint::Percentage(50), // Recent activity
        ])
        .split(area);

    // Instance Statistics
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title(" Instance Statistics ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .border_style(Style::default().fg(Color::Cyan));

    let stats_content = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Stats: Total Playtime: ", Style::default().fg(Color::Yellow)),
            Span::styled("2h 45m", Style::default().fg(Color::White).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last:  Last Played: ", Style::default().fg(Color::Yellow)),
            Span::styled("Never", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Size: Instance Size: ", Style::default().fg(Color::Yellow)),
            Span::styled("~847 MB", Style::default().fg(Color::White).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Created:  Created: ", Style::default().fg(Color::Yellow)),
            Span::styled("Today", Style::default().fg(Color::Green)),
        ]),
    ])
    .block(stats_block)
    .wrap(Wrap { trim: true });

    f.render_widget(stats_content, info_chunks[0]);

    // Recent Activity / Quick Info
    let activity_block = Block::default()
        .borders(Borders::ALL)
        .title(" Quick Info ")
        .title_style(Style::default().fg(Color::Magenta).bold())
        .border_style(Style::default().fg(Color::Magenta));

    let activity_content = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Performance: Performance: ", Style::default().fg(Color::Green)),
            Span::styled("Optimized", Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Java: Java Version: ", Style::default().fg(Color::Blue)),
            Span::styled("Auto-detected", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mode: Game Mode: ", Style::default().fg(Color::Yellow)),
            Span::styled("Survival", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Network: Multiplayer: ", Style::default().fg(Color::Cyan)),
            Span::styled("Ready", Style::default().fg(Color::Green)),
        ]),
    ])
    .block(activity_block)
    .wrap(Wrap { trim: true });

    f.render_widget(activity_content, info_chunks[1]);
}

/// Render instance mods tab
fn render_instance_mods(f: &mut Frame, area: Rect, instance_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Mod list
            Constraint::Percentage(40), // Mod actions/info
        ])
        .split(area);

    // Left side: Mod list (placeholder)
    let mods_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Installed Mods ({}) ", 0))
        .title_style(Style::default().fg(Color::Green).bold())
        .border_style(Style::default().fg(Color::Green));

    let mods_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Java: Mod Management Coming Soon!", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from("Planned features:"),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Browse & install from Modrinth"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Enable/disable mods easily"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Automatic dependency resolution"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Mod compatibility checking"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("One-click mod updates"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Stay tuned for updates! Launch", Style::default().fg(Color::Cyan)),
        ]),
    ])
    .block(mods_block)
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    f.render_widget(mods_content, chunks[0]);

    // Right side: Actions panel
    let actions_block = Block::default()
        .borders(Borders::ALL)
        .title(" Mod Actions ")
        .title_style(Style::default().fg(Color::Blue).bold())
        .border_style(Style::default().fg(Color::Blue));

    let actions_content = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Browse Browse Mods", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from("Discover new mods"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Performance: Quick Install", Style::default().fg(Color::Green).bold()),
        ]),
        Line::from("Install popular mods"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Update Update All", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from("Check for mod updates"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Import  Import Pack", Style::default().fg(Color::Magenta).bold()),
        ]),
        Line::from("Import modpack files"),
        Line::from(""),
        Line::from(vec![
            Span::styled("WIP  Work in Progress", Style::default().fg(Color::Red)),
        ]),
    ])
    .block(actions_block)
    .wrap(Wrap { trim: true });

    f.render_widget(actions_content, chunks[1]);
}

/// Render instance settings tab
fn render_instance_settings(f: &mut Frame, area: Rect, selected_index: usize, instance: &crate::tui::app::Instance) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Settings options
            Constraint::Percentage(50), // Settings details
        ])
        .split(area);

    // Left side: Settings options
    let settings_block = Block::default()
        .borders(Borders::ALL)
        .title(" Instance Management ")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .border_style(Style::default().fg(Color::Yellow));

    let settings_items = vec![
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  [R] ", Style::default().fg(Color::Green).bold()),
                Span::styled("Rename Instance", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled("Change the instance name", Style::default().fg(Color::Gray)),
            ]),
        ]).style(if selected_index == 0 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
        
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  [J] ", Style::default().fg(Color::Blue).bold()),
                Span::styled("Java Settings", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled("Configure JVM and memory", Style::default().fg(Color::Gray)),
            ]),
        ]).style(if selected_index == 1 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
        
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  [L] ", Style::default().fg(Color::Cyan).bold()),
                Span::styled("Launch Options", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled("Game arguments and JVM flags", Style::default().fg(Color::Gray)),
            ]),
        ]).style(if selected_index == 2 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
        
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("  [X] ", Style::default().fg(Color::Red).bold()),
                Span::styled("Delete Instance", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled("Permanently remove this instance", Style::default().fg(Color::Gray)),
            ]),
        ]).style(if selected_index == 3 {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        }),
    ];

    let settings_list = List::new(settings_items)
        .block(settings_block)
        .highlight_symbol("  ");
    
    // Create a ListState for scrolling
    let mut list_state = ListState::default();
    list_state.select(Some(selected_index));
    
    f.render_stateful_widget(settings_list, chunks[0], &mut list_state);

    // Right side: Settings details
    let details_block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration Panel ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .border_style(Style::default().fg(Color::Cyan));

    let details_content = match selected_index {
        0 => Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("[R] Rename Instance", Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current Name: ", Style::default().fg(Color::Yellow)),
                Span::styled(&instance.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(""),
            Line::from("This feature is under development."),
            Line::from(""),
            Line::from("Will allow you to:"),
            Line::from("  > Change instance display name"),
            Line::from("  > Keep all mods and settings"),
            Line::from("  > Update folder name if needed"),
        ]),
        1 => Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("[J] Java Configuration", Style::default().fg(Color::Blue).bold()),
            ]),
            Line::from(""),
            Line::from("Java settings for this instance:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().fg(Color::Yellow)),
                Span::styled("Auto-detected", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Java Path: ", Style::default().fg(Color::Yellow)),
                Span::styled("System default", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("Coming soon:"),
            Line::from("  > Memory allocation slider"),
            Line::from("  > Custom Java path selection"),
            Line::from("  > JVM argument editor"),
        ]),
        2 => Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("[L] Launch Options", Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(""),
            Line::from("Launch configuration for this instance:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::Yellow)),
                Span::styled(&instance.version, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Loader: ", Style::default().fg(Color::Yellow)),
                Span::styled(&instance.loader, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from("Coming soon:"),
            Line::from("  > Custom game arguments"),
            Line::from("  > Resolution settings"),
            Line::from("  > Server auto-connect"),
        ]),
        3 => Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("[X] Delete Instance", Style::default().fg(Color::Red).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("!!! WARNING !!!", Style::default().fg(Color::Red).bold()),
            ]),
            Line::from(""),
            Line::from("This will permanently delete:"),
            Line::from(vec![
                Span::raw("  > Instance: "),
                Span::styled(&instance.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from("  > All world saves"),
            Line::from("  > All mods and configurations"),
            Line::from("  > All resource packs and shaders"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press Enter to confirm deletion", Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            if instance.is_running {
                Line::from(vec![
                    Span::styled("ERROR: Cannot delete running instance", Style::default().fg(Color::Red).bold()),
                ])
            } else {
                Line::from(vec![
                    Span::styled("READY: Instance can be deleted", Style::default().fg(Color::Green)),
                ])
            },
        ]),
        _ => Paragraph::new(vec![
            Line::from(""),
            Line::from("Select a configuration option"),
        ]),
    }
    .block(details_block)
    .wrap(Wrap { trim: true });

    f.render_widget(details_content, chunks[1]);
}

/// Render instance logs tab
fn render_instance_logs(f: &mut Frame, area: Rect, instance_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Log controls
            Constraint::Min(0),    // Log viewer
        ])
        .split(area);

    // Top: Log controls
    let controls_block = Block::default()
        .borders(Borders::ALL)
        .title(" Log Controls ")
        .title_style(Style::default().fg(Color::Green).bold())
        .border_style(Style::default().fg(Color::Green));

    let controls_content = Paragraph::new(
        Line::from(vec![
            Span::styled("Filter: Filter: ", Style::default().fg(Color::Yellow)),
            Span::raw("All  "),
            Span::styled("Export Export  ", Style::default().fg(Color::Blue)),
            Span::styled("Clear  Clear  ", Style::default().fg(Color::Red)),
            Span::styled("Live Live  ", Style::default().fg(Color::Green)),
            Span::styled("Pause  Pause", Style::default().fg(Color::Gray)),
        ])
    )
    .block(controls_block)
    .alignment(Alignment::Center);

    f.render_widget(controls_content, chunks[0]);

    // Bottom: Log viewer
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Logs for {} ", instance_name))
        .title_style(Style::default().fg(Color::Cyan).bold())
        .border_style(Style::default().fg(Color::Cyan));

    let logs_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("📋 Real-time Log Viewer", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from("This feature is under development!"),
        Line::from(""),
        Line::from("Planned features:"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Real-time game log streaming"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Error highlighting and filtering"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Performance metrics display"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Log export and sharing"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Crash report analysis"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::raw("Search and navigation"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Monitor Monitor your game's performance in real-time!", Style::default().fg(Color::Cyan)),
        ]),
    ])
    .block(logs_block)
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    f.render_widget(logs_content, chunks[1]);
}

/// Get help text for instance settings
fn get_instance_settings_help(_app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══ INSTANCE SETTINGS ═══", Style::default().fg(Color::Cyan).bold())
        ]),
        Line::from("Left/Right or h/l   Switch between sub-tabs"),
        Line::from("Up/Down or j/k      Navigate items/actions"),
        Line::from("Enter               Select action/item"),
        Line::from("Esc                 Return to instances"),
        Line::from(""),
        Line::from("Overview Tab:"),
        Line::from("  ↑/↓               Navigate between Play/Kill/Folder"),
        Line::from("  Enter on Play     Launch the instance"),
        Line::from("  Enter on Kill     Stop running instance"),
        Line::from("  Enter on Folder   Open instance directory"),
        Line::from(""),
        Line::from("Other tabs show planned features."),
        Line::from(""),
    ]
}
