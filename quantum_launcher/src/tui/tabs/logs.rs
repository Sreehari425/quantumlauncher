// QuantumLauncher TUI - Logs Tab

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the logs tab
pub fn render_logs_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Game Logs ");
    
    let log_text = if app.game_logs.is_empty() {
        "No logs available.\n\nLogs will appear here when you launch a Minecraft instance.\nPress 'c' to clear logs when available.".to_string()
    } else {
        app.game_logs.join("\n")
    };
    
    let paragraph = Paragraph::new(log_text)
        .block(block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
