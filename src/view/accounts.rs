use crate::app::{App, ClickAction, InputMode};
use crate::message::Message;
use crate::theme::ui;
use crate::view::{SELECTED_PREFIX, UNSELECTED_PREFIX, render_footer_bar, render_scrollbar};
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
    render_account_list(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &mut App, frame: &mut Frame, area: Rect) {
    let back_text = "[Esc] Back";
    let back_x_offset = "Select Account".len() + 2; // title + "  "
    let mut spans = vec![
        Span::styled("Select Account", Style::default().fg(ui::PRIMARY).bold()),
        Span::raw("  "),
        Span::styled(back_text, Style::default().fg(ui::MUTED)),
    ];

    // Register click region for back button (area.x + 1 for border + offset)
    let back_region = Rect {
        x: area.x + 1 + back_x_offset as u16,
        y: area.y,
        width: back_text.len() as u16,
        height: area.height,
    };
    app.register_click(back_region, ClickAction::GoBack);

    // Show search query if active
    if !app.search_query.is_empty() || app.input_mode == InputMode::Search {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("/", Style::default().fg(ui::HIGHLIGHT)));
        spans.push(Span::styled(
            &app.input_buffer,
            Style::default().fg(ui::HIGHLIGHT),
        ));
        if app.input_mode == InputMode::Search {
            spans.push(Span::styled("_", Style::default().fg(ui::HIGHLIGHT)));
        }
    }

    let header = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_account_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let filtered_set: std::collections::HashSet<usize> =
        app.filtered_account_indices.iter().copied().collect();

    let items: Vec<ListItem> = app
        .accounts
        .iter()
        .enumerate()
        .filter(|(idx, _)| filtered_set.contains(idx))
        .map(|(idx, account)| {
            let is_selected = idx == app.selected_account_index;
            let is_active = app
                .active_account
                .as_ref()
                .map(|a| a.profile_id == account.profile_id)
                .unwrap_or(false);

            let prefix = if is_selected {
                SELECTED_PREFIX
            } else {
                UNSELECTED_PREFIX
            };
            let active_marker = if is_active { "[*]" } else { "[ ]" };

            let style = if is_selected {
                Style::default()
                    .fg(ui::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    active_marker,
                    if is_active {
                        Style::default().fg(ui::ACTIVE)
                    } else {
                        Style::default().fg(ui::MUTED)
                    },
                ),
                Span::raw(" "),
                Span::styled(&account.username, style),
            ]))
        })
        .collect();

    let title = if !app.search_query.is_empty() {
        format!(
            "Accounts ({}/{})",
            app.filtered_account_count(),
            app.accounts.len()
        )
    } else {
        "Accounts".to_string()
    };

    let total_items = items.len();

    let list = if items.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No matches. Press Esc to clear search."
        } else {
            "No accounts found. Add accounts in PrismLauncher."
        };
        List::new(vec![ListItem::new(Span::styled(
            format!("  {}", msg),
            Style::default().fg(ui::MUTED),
        ))])
    } else {
        List::new(items)
    }
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);

    // Register click regions for each visible account item
    let account_indices: Vec<usize> = app.filtered_account_indices.clone();
    for (row_offset, idx) in account_indices.iter().enumerate() {
        let row_y = area.y + 1 + row_offset as u16;
        if row_y >= area.y + area.height.saturating_sub(1) {
            break;
        }
        let row_rect = Rect {
            x: area.x,
            y: row_y,
            width: area.width,
            height: 1,
        };
        app.register_click(row_rect, ClickAction::SelectItem(*idx));
    }

    // Scrollbar
    let selected_pos = app
        .filtered_account_indices
        .iter()
        .position(|&idx| idx == app.selected_account_index)
        .unwrap_or(0);
    render_scrollbar(
        frame,
        area,
        total_items,
        inner_height,
        selected_pos.saturating_sub(inner_height / 2),
    );
}

fn render_footer(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.input_mode == InputMode::Search {
        let keys: &[(&str, &str, Option<Message>)] = &[
            ("Type", "Search", None),
            ("Enter", "Confirm", Some(Message::SearchConfirm)),
            ("Esc", "Cancel", Some(Message::SearchCancel)),
        ];
        render_footer_bar(app, frame, area, keys);
    } else {
        let keys: &[(&str, &str, Option<Message>)] = &[
            ("j/k", "Nav", None),
            ("l/Enter", "Select", Some(Message::ConfirmAccountSelection)),
            ("/", "Search", Some(Message::StartSearch)),
            ("h/Esc", "Back", Some(Message::Back)),
        ];
        render_footer_bar(app, frame, area, keys);
    }
}
