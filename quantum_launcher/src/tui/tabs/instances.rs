// QuantumLauncher TUI - Instances Tab

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the instances tab
pub fn render_instances_tab(f: &mut Frame, area: Rect, app: &mut App) {
    if app.instances.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Instances ");
        let paragraph = Paragraph::new("No instances found.\nPress F5 to refresh or go to Create tab to make a new instance.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .instances
        .iter()
        .map(|instance| {
            ListItem::new(Line::from(vec![
                Span::raw(&instance.name),
                Span::styled(format!(" ({})", instance.version), Style::default().fg(Color::Gray)),
                Span::styled(format!(" [{}]", instance.loader), Style::default().fg(Color::Green)),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_instance));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Instances ")
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut list_state);
}
