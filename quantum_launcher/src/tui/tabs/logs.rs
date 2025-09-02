// QuantumLauncher TUI - Logs Tab (advanced, migrated from ui.rs)

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the global logs tab with viewport and auto-follow behavior
pub fn render_logs_tab(f: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Game Logs ");

    // Update visible lines estimate based on area height (minus borders and title spacing)
    let inner_height = area.height.saturating_sub(4).max(1); // leave space for title and padding
    app.logs_visible_lines = inner_height as usize;

    let total = app.game_logs.len();
    // Compute start index based on offset from bottom
    let end = total;
    // Maximum offset to keep at least a full page when possible
    let max_offset = total.saturating_sub(app.logs_visible_lines);
    if app.logs_offset > max_offset {
        app.logs_offset = max_offset;
    }
    let start = end.saturating_sub(app.logs_visible_lines + app.logs_offset);

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
        app.game_logs[start..end]
            .iter()
            .cloned()
            .map(Line::from)
            .collect::<Vec<_>>()
    };

    let paragraph = Paragraph::new(log_text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render the per-instance logs tab with viewport and auto-follow
pub fn render_instance_logs(
    f: &mut Frame,
    area: Rect,
    app: &mut App,
    instance_name: &str,
) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::text::Span;

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
        .border_style(Style::default().fg(Color::Green));

    let controls_content = Paragraph::new(
        vec![
            Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(Color::Yellow)),
                Span::raw("All  "),
                Span::styled("Export  ", Style::default().fg(Color::Blue)),
                Span::styled("Clear  ", Style::default().fg(Color::Red)),
                Span::styled("Live  ", Style::default().fg(Color::Green)),
                Span::styled("Pause", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Scroll: ", Style::default().fg(Color::Yellow)),
                Span::raw("↑/↓, PageUp/PageDown, Home/End, Mouse wheel | "),
                Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
                Span::raw("Auto-follow at bottom; scroll up to pause"),
            ]),
        ],
    )
    .block(controls_block);

    f.render_widget(controls_content, chunks[0]);

    // Bottom: Log viewer
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Logs for {} ", instance_name))
        .border_style(Style::default().fg(Color::Cyan));

    // Compute visible lines for instance logs viewport
    let inner_height = chunks[1].height.saturating_sub(2).max(1); // borders only
    app.instance_logs_visible_lines = inner_height as usize;

    // Determine which slice of the instance buffer to show based on offset
    let content_lines: Vec<Line> = if let Some(buf) = app.instance_logs.get(instance_name) {
        let total = buf.len();
        // Clamp offset based on total and viewport
        let max_offset = total.saturating_sub(app.instance_logs_visible_lines);
        if app.instance_logs_offset > max_offset {
            app.instance_logs_offset = max_offset;
        }
        let end = total;
        let start = end.saturating_sub(app.instance_logs_visible_lines + app.instance_logs_offset);
        if total == 0 {
            vec![
                Line::from(""),
                Line::from("No logs yet. Launch the instance to see output here."),
            ]
        } else {
            buf[start..end]
                .iter()
                .map(|s| Line::from(s.clone()))
                .collect()
        }
    } else {
        vec![
            Line::from(""),
            Line::from("No logs yet. Launch the instance to see output here."),
        ]
    };

    let logs_content = Paragraph::new(content_lines)
        .block(logs_block)
        .wrap(Wrap { trim: true });

    f.render_widget(logs_content, chunks[1]);
}
