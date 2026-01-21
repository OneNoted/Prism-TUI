use crate::app::{App, ClickAction, InputMode, LogLevel, LogSource};
use crate::message::Message;
use crate::theme::ui;
use crate::view::{
    SELECTED_PREFIX, UNSELECTED_PREFIX, render_footer_bar, render_scrollbar, truncate,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &mut App, frame: &mut Frame, area: Rect) {
    let title = match app.log_source {
        LogSource::Instance => {
            if let Some(instance) = app.selected_instance() {
                format!("Logs: {}", instance.name)
            } else {
                "Logs: Instance".to_string()
            }
        }
        LogSource::Launcher => "Logs: Launcher".to_string(),
    };

    let mut spans = vec![Span::styled(title, Style::default().fg(ui::PRIMARY).bold())];

    // Show log search if active
    if !app.log_search_query.is_empty() || app.input_mode == InputMode::LogSearch {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("/", Style::default().fg(ui::HIGHLIGHT)));
        spans.push(Span::styled(
            &app.log_search_query,
            Style::default().fg(ui::HIGHLIGHT),
        ));
        if app.input_mode == InputMode::LogSearch {
            spans.push(Span::styled("_", Style::default().fg(ui::HIGHLIGHT)));
        }
        if !app.log_search_matches.is_empty() {
            spans.push(Span::styled(
                format!(
                    " ({}/{})",
                    app.log_search_current + 1,
                    app.log_search_matches.len()
                ),
                Style::default().fg(ui::MUTED),
            ));
        }
    }

    // Show active log level filters
    if !app.log_level_filter.is_empty() {
        spans.push(Span::raw("  "));
        let filter_text: Vec<&str> = [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
        ]
        .iter()
        .filter(|l| app.log_level_filter.contains(l))
        .map(|l| l.label())
        .collect();
        spans.push(Span::styled(
            format!("[{}]", filter_text.join(",")),
            Style::default().fg(ui::WARNING),
        ));
    }

    let header = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_content(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split into file list (30%) and content preview (70%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_file_list(app, frame, chunks[0]);
    render_log_preview(app, frame, chunks[1]);
}

fn render_file_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .log_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = idx == app.selected_log_index;
            let prefix = if is_selected {
                SELECTED_PREFIX
            } else {
                UNSELECTED_PREFIX
            };

            let style = if is_selected {
                Style::default()
                    .fg(ui::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(truncate(&entry.name, 20), style),
                Span::raw(" "),
                Span::styled(entry.formatted_size(), Style::default().fg(ui::MUTED)),
            ]))
        })
        .collect();

    let total_items = items.len();
    let title = format!("Files ({})", app.log_entries.len());
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);

    // Register click regions for each visible log file item
    for idx in 0..app.log_entries.len() {
        let row_y = area.y + 1 + idx as u16;
        if row_y >= area.y + area.height.saturating_sub(1) {
            break;
        }
        let row_rect = Rect {
            x: area.x,
            y: row_y,
            width: area.width,
            height: 1,
        };
        app.register_click(row_rect, ClickAction::SelectLogFile(idx));
    }

    // Scrollbar
    render_scrollbar(
        frame,
        area,
        total_items,
        inner_height,
        app.selected_log_index.saturating_sub(inner_height / 2),
    );
}

fn render_log_preview(app: &mut App, frame: &mut Frame, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;

    let filtered_content = app.filtered_log_content();
    let total_lines = filtered_content.len();

    let search_match_set: std::collections::HashSet<usize> =
        app.log_search_matches.iter().copied().collect();

    let visible_lines: Vec<Line> = filtered_content
        .iter()
        .skip(app.log_scroll_offset)
        .take(inner_height)
        .map(|(original_idx, line)| {
            let is_search_match = search_match_set.contains(original_idx);

            // Basic log level highlighting
            let mut style = if line.contains("ERROR") || line.contains("[ERROR]") {
                Style::default().fg(ui::ERROR)
            } else if line.contains("WARN") || line.contains("[WARN]") {
                Style::default().fg(ui::WARNING)
            } else if line.contains("INFO") || line.contains("[INFO]") {
                Style::default().fg(ui::INFO)
            } else if line.contains("DEBUG") || line.contains("[DEBUG]") {
                Style::default().fg(ui::DEBUG)
            } else {
                Style::default()
            };

            if is_search_match {
                style = style.bg(ui::HIGHLIGHT).fg(Color::Black);
            }

            Line::from(Span::styled(line.as_str(), style))
        })
        .collect();

    let title = if app.log_content.is_empty() {
        "Preview (press Enter to load)".to_string()
    } else {
        format!(
            "Preview ({}-{}/{})",
            app.log_scroll_offset + 1,
            (app.log_scroll_offset + inner_height).min(total_lines),
            total_lines
        )
    };

    let preview =
        Paragraph::new(visible_lines).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(preview, area);

    // Register the preview area for scroll targeting
    app.register_click(area, ClickAction::ScrollLogPreview);

    // Scrollbar for preview
    render_scrollbar(
        frame,
        area,
        total_lines,
        inner_height,
        app.log_scroll_offset,
    );
}

fn render_footer(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.input_mode == InputMode::LogSearch {
        let keys: &[(&str, &str, Option<Message>)] = &[
            ("Type", "Search", None),
            ("Enter", "Confirm", Some(Message::LogSearchConfirm)),
            ("Esc", "Cancel", Some(Message::LogSearchCancel)),
        ];
        render_footer_bar(app, frame, area, keys);
    } else {
        let keys: &[(&str, &str, Option<Message>)] = &[
            ("j/k", "Nav", None),
            ("l/Enter", "Load", Some(Message::LoadLogContent)),
            ("J/K", "Scroll", None),
            ("/", "Search", Some(Message::StartLogSearch)),
            ("n/N", "Next/Prev", None),
            ("1-4", "Filter", None),
            ("0", "All", Some(Message::ShowAllLogLevels)),
            ("e", "Editor", Some(Message::OpenLogInEditor)),
            ("o", "Folder", Some(Message::OpenLogFolder)),
            ("h/Esc", "Back", Some(Message::Back)),
        ];
        render_footer_bar(app, frame, area, keys);
    }
}
