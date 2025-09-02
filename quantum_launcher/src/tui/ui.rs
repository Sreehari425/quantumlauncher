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
use crate::tui::tabs::logs::{render_logs_tab, render_instance_logs};
use crate::tui::tabs::instances::render_instances_tab;
use crate::tui::tabs::create::render_create_tab;
use crate::tui::tabs::accounts::render_accounts_tab;
use crate::tui::tabs::settings::render_settings_tab;

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

/// Render the header with tabs
fn render_header(f: &mut Frame, area: Rect, app: &App) {
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
        TabId::InstanceSettings => 0,
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

/// Render the footer with status and help
fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(35), Constraint::Length(20)])
        .split(area);

    let status = Paragraph::new(app.status_message.clone())
        .block(Block::default().borders(Borders::ALL).title(" Status "))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    let account_info = if let Some(current_account) = &app.current_account {
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

    let progress_area = Rect { x: area.x + 2, y: area.y + 3, width: area.width - 4, height: 1 };
    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(0.5);
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

    f.render_widget(Clear, popup_area);
    f.render_widget(para, popup_area);
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

/// Help for About tab
// About help removed; integrated into Settings help

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

    // Version search controls
    help.extend(vec![
        Line::from(vec![
            Span::styled("Version search:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("Ctrl+S             Toggle search mode for versions"),
        Line::from("Type               Filter versions while search is active"),
        Line::from("Backspace          Delete character (search mode)"),
        Line::from("Esc                Exit search mode"),
        Line::from("Enter              Create using the selected match"),
        Line::from(""),
    ]);

    // Version type filters
    help.extend(vec![
        Line::from(vec![
            Span::styled("Version filters:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("F6                 Toggle Release"),
        Line::from("F7                 Toggle Snapshot"),
        Line::from("F8                 Toggle Beta"),
        Line::from("F9                 Toggle Alpha"),
        Line::from("F10                Reset all filters"),
        Line::from(""),
    ]);

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
            Span::styled("Filters:", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("Ctrl+R             Toggle Release"),
        Line::from("Ctrl+P             Toggle Snapshot"),
        Line::from("Ctrl+B             Toggle Beta"),
        Line::from("Ctrl+A             Toggle Alpha"),
        Line::from("Ctrl+0             Reset all filters"),
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
        Line::from("↑/↓ or j/k         Navigate left menu / license list"),
        Line::from("→ / Enter          Focus middle pane when on 'Licenses'"),
        Line::from("← / Esc            Return focus to left pane"),
        Line::from("PageUp/PageDown    Scroll license text (right pane)"),
        Line::from("Mouse wheel        Scroll license text (right pane)"),
        Line::from("r                  Reset to defaults (coming soon)"),
        Line::from("s                  Save settings (coming soon)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Licenses page: left = category, middle = license, right = text")
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

// accounts rendering moved to tabs::accounts

/// Render the logs tab
// moved to tabs::logs::render_logs_tab

/// Help for Logs tab
fn get_logs_help() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══ LOGS TAB ═══", Style::default().fg(Color::Magenta).bold())
        ]),
        Line::from("↑/↓ or j/k         Scroll by one line"),
        Line::from("PageUp/PageDown    Scroll by one page"),
        Line::from("Home/End or g/G    Jump to top/bottom"),
        Line::from("Mouse wheel        Scroll logs"),
        Line::from("c                  Clear logs"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Logs auto-follow at bottom; scroll up to pause follow")
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
                crate::tui::app::InstanceSettingsTab::Logs => {
                    let instance_name = instance.name.clone();
                    render_instance_logs(f, chunks[2], app, &instance_name)
                },
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
fn render_instance_mods(f: &mut Frame, area: Rect, _instance_name: &str) {
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
// moved to tabs::logs::render_instance_logs

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
