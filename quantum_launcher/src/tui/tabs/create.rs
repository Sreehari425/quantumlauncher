// QuantumLauncher TUI - Create Instance Tab

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::app::App;

/// Render the create instance tab
pub fn render_create_tab(f: &mut Frame, area: Rect, app: &mut App) {
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
            .title(" Instance Name ")
    );

    f.render_widget(name_input, chunks[0]);

    // Version list (supports live search)
    if app.available_versions.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Minecraft Versions ");
        let paragraph = Paragraph::new("Loading versions...\nPress F5 to refresh")
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let version_source = if app.version_search_active {
            &app.filtered_versions
        } else {
            &app.available_versions
        };

        // Build title with search hint
        let mut title = String::from(" Minecraft Versions ");
        if app.version_search_active {
            title.push_str(" - Search: ");
            title.push_str(&app.version_search_query);
        } else {
            title.push_str("  (type to search)");
        }

        let items: Vec<ListItem> = version_source
            .iter()
            .map(|version| {
                ListItem::new(Line::from(vec![
                    Span::raw(&version.name),
                    Span::styled(" [Release]", Style::default().fg(Color::Cyan)), // Default to Release
                ]))
            })
            .collect();

        let mut list_state = ListState::default();
        let sel = if app.version_search_active {
            app.selected_filtered_version
        } else {
            app.selected_version
        };
        list_state.select(Some(sel));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}
