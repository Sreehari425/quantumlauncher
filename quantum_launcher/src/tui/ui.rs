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
        render_help_popup(f);
    }
}

/// Render the header with tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let tabs = vec![
        Line::from("Instances (i)"),
        Line::from("Create (c)"),
        Line::from("Settings (s)"),
        Line::from("Accounts (a)"),
    ];

    let selected_tab = match app.current_tab {
        TabId::Instances => 0,
        TabId::Create => 1,
        TabId::Settings => 2,
        TabId::Accounts => 3,
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
            let content = format!("{} [{}]{}", account.username, account.account_type, status);
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
    if app.is_login_mode {
        // Login form
        let login_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Username input
                Constraint::Length(3), // Password input
                Constraint::Length(2), // Instructions
            ])
            .split(chunks[1]);

        let username_input = Paragraph::new(app.login_username.as_str())
            .block(Block::default().borders(Borders::ALL).title("Username"))
            .style(Style::default().fg(Color::White));
        f.render_widget(username_input, login_chunks[0]);

        let password_display = "*".repeat(app.login_password.len());
        let password_input = Paragraph::new(password_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(Style::default().fg(Color::White));
        f.render_widget(password_input, login_chunks[1]);

        let instructions = Paragraph::new("Press Enter to login, Esc to cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, login_chunks[2]);
    } else {
        // Account actions
        let selected_account_info = if let Some(account) = app.get_selected_account() {
            if account.is_logged_in {
                format!("Selected: {} (logged in)\nPress 'l' to logout, 'n' to add new account", account.username)
            } else {
                format!("Selected: {} (not logged in)\nPress 'l' to login, 'n' to add new account", account.username)
            }
        } else {
            "No accounts available\nPress 'n' to add new account".to_string()
        };

        let account_info = Paragraph::new(selected_account_info)
            .block(Block::default().borders(Borders::ALL).title("Actions"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(account_info, chunks[1]);
    }
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

/// Render help popup with all controls
fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("QuantumLauncher TUI Controls", Style::default().fg(Color::Yellow).bold())
        ]),
        Line::from(""),
        Line::from("═══ NAVIGATION ═══"),
        Line::from("↑/↓ or j/k         Navigate up/down in lists"),
        Line::from("←/→ or h/l         Switch between tabs"),
        Line::from("Enter              Select/activate current item"),
        Line::from(""),
        Line::from("═══ TAB SHORTCUTS ═══"),
        Line::from("i                  Go to Instances tab"),
        Line::from("c                  Go to Create tab"),
        Line::from("s                  Go to Settings tab"),
        Line::from("a                  Go to Accounts tab"),
        Line::from(""),
        Line::from("═══ INSTANCE TAB ═══"),
        Line::from("Enter              Launch selected instance (coming soon)"),
        Line::from("F5                 Refresh instance list"),
        Line::from(""),
        Line::from("═══ CREATE TAB ═══"),
        Line::from("n                  Toggle instance name editing"),
        Line::from("↑/↓                Navigate version list"),
        Line::from("Enter              Create instance (coming soon)"),
        Line::from("Backspace          Delete character (when editing name)"),
        Line::from(""),
        Line::from("═══ ACCOUNTS TAB ═══"),
        Line::from("l                  Login/logout selected account"),
        Line::from("n                  Add new account (coming soon)"),
        Line::from("↑/↓                Navigate account list"),
        Line::from("Esc                Cancel login mode"),
        Line::from(""),
        Line::from("═══ GENERAL ═══"),
        Line::from("?                  Show/hide this help popup"),
        Line::from("q or Esc           Quit application"),
        Line::from("F5                 Refresh data"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press '?' or Esc to close this help", Style::default().fg(Color::Green).italic())
        ]),
    ];

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
