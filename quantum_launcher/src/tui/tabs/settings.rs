// QuantumLauncher TUI - Settings Tab

use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the settings tab
pub fn render_settings_tab(f: &mut Frame, area: Rect, _app: &App) {
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
