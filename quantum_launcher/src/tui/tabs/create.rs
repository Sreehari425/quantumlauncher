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
            .title(" Instance Name ")
    );

    f.render_widget(name_input, chunks[0]);

    // Version list
    if app.available_versions.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Minecraft Versions ");
        let paragraph = Paragraph::new("Loading versions...\nPress F5 to refresh")
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .available_versions
            .iter()
            .map(|version| {
                ListItem::new(Line::from(vec![
                    Span::raw(&version.name),
                    Span::styled(" [Release]", Style::default().fg(Color::Cyan)), // Default to Release
                ]))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_version));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Minecraft Versions ")
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}
