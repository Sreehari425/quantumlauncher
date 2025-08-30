// QuantumLauncher TUI - Main UI Renderer

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::tui::{
    app::App,
    widgets::{render_header, render_footer, render_loading_popup, render_help_popup},
    tabs::{render_instances_tab, render_create_tab, render_settings_tab, render_accounts_tab, render_logs_tab},
};

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

/// Render the main content area based on current tab
fn render_main_content(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    use crate::tui::app::TabId;
    
    match app.current_tab {
        TabId::Instances => render_instances_tab(f, area, app),
        TabId::Create => render_create_tab(f, area, app),
        TabId::Settings => render_settings_tab(f, area, app),
        TabId::Accounts => render_accounts_tab(f, area, app),
        TabId::Logs => render_logs_tab(f, area, app),
    }
}
