use crate::app::{App, ClickAction, InputMode, VisualRow};
use crate::message::Message;
use crate::theme::ui;
use crate::view::{
    SELECTED_PREFIX, UNSELECTED_PREFIX, render_footer_bar, render_scrollbar, truncate,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

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
    render_instance_table(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &mut App, frame: &mut Frame, area: Rect) {
    let account_text = app
        .active_account
        .as_ref()
        .map(|a| format!("[Account: {}]", a.username))
        .unwrap_or_else(|| "[No Account]".to_string());

    let sort_text = format!(
        "[Sort: {} {}]",
        app.sort_mode.label(),
        if app.sort_ascending { "▲" } else { "▼" }
    );

    let mut spans = vec![
        Span::styled("Prism-TUI", Style::default().fg(ui::PRIMARY).bold()),
        Span::raw(" "),
        Span::styled(account_text, Style::default().fg(ui::ACTIVE)),
        Span::raw(" "),
        Span::styled(sort_text, Style::default().fg(ui::MUTED)),
    ];

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

fn render_instance_table(app: &mut App, frame: &mut Frame, area: Rect) {
    let width = area.width;
    let inner_height = area.height.saturating_sub(2) as usize;

    let visual = app.visual_rows();
    let mut rows: Vec<Row> = Vec::new();
    let mut selected_row: Option<usize> = None;
    let selected_group_key = app.selected_group_key();

    for (row_idx, vrow) in visual.iter().enumerate() {
        match vrow {
            VisualRow::GroupHeader {
                key: _,
                collapsed,
                count,
            } => {
                let indicator = if *collapsed { "[+]" } else { "[-]" };
                // Recover group name from the key (which is the display name)
                let group_name = match vrow {
                    VisualRow::GroupHeader { key, .. } => key.as_str(),
                    _ => unreachable!(),
                };
                let is_selected_group = selected_group_key.as_deref() == Some(group_name);
                let prefix = if is_selected_group { ">" } else { " " };
                let header_text = format!("{} {} {} ({})", prefix, indicator, group_name, count);
                let style = if is_selected_group {
                    Style::default()
                        .fg(ui::PRIMARY)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(ui::HIGHLIGHT)
                        .add_modifier(Modifier::BOLD)
                };
                rows.push(Row::new(vec![Cell::from(Span::styled(header_text, style))]).height(1));
            }
            VisualRow::Instance(visual_idx) => {
                let instance = match app.instance_by_visual_idx(*visual_idx) {
                    Some(i) => i,
                    None => continue,
                };

                let is_running = app.is_instance_running(&instance.id);
                let is_selected = *visual_idx == app.selected_instance_index;
                let prefix = if is_selected {
                    SELECTED_PREFIX
                } else {
                    UNSELECTED_PREFIX
                };

                if is_selected {
                    selected_row = Some(row_idx);
                }

                let style = if is_selected {
                    Style::default()
                        .fg(ui::PRIMARY)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let muted = Style::default().fg(ui::MUTED);
                let active_style = Style::default().fg(ui::ACTIVE);

                let join_indicator = instance
                    .server_join
                    .as_ref()
                    .filter(|sj| sj.enabled)
                    .map(|sj| sj.address.as_str())
                    .unwrap_or("");

                let running_prefix = if is_running { "● " } else { "" };

                let name_cell = |max_len: usize| -> Cell<'_> {
                    if is_running {
                        Cell::from(Line::from(vec![
                            Span::styled(prefix, style),
                            Span::styled("● ", Style::default().fg(ui::ACTIVE)),
                            Span::styled(
                                truncate(
                                    &instance.name,
                                    max_len.saturating_sub(running_prefix.len()),
                                ),
                                style,
                            ),
                        ]))
                    } else {
                        Cell::from(Span::styled(
                            format!("{}{}", prefix, truncate(&instance.name, max_len)),
                            style,
                        ))
                    }
                };

                let cells = if width < 60 {
                    vec![name_cell((width as usize).saturating_sub(6))]
                } else if width < 80 {
                    vec![
                        name_cell(25),
                        Cell::from(Span::styled(
                            truncate(&instance.minecraft_version, 12),
                            muted,
                        )),
                    ]
                } else if width < 100 {
                    vec![
                        name_cell(25),
                        Cell::from(Span::styled(
                            truncate(&instance.minecraft_version, 12),
                            muted,
                        )),
                        Cell::from(Span::styled(instance.formatted_playtime(), muted)),
                    ]
                } else {
                    vec![
                        name_cell(25),
                        Cell::from(Span::styled(
                            truncate(&instance.minecraft_version, 12),
                            muted,
                        )),
                        Cell::from(Span::styled(
                            instance.mod_loader.as_deref().unwrap_or("-"),
                            muted,
                        )),
                        Cell::from(Span::styled(instance.formatted_playtime(), muted)),
                        Cell::from(Span::styled(truncate(join_indicator, 20), active_style)),
                    ]
                };

                rows.push(Row::new(cells).height(1));
            }
        }
    }

    let total_visible = rows.len();

    if rows.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No matches. Press Esc to clear search."
        } else {
            "No instances found. Add instances in PrismLauncher."
        };
        rows.push(
            Row::new(vec![Cell::from(Span::styled(
                format!("  {}", msg),
                Style::default().fg(ui::MUTED),
            ))])
            .height(1),
        );
    }

    let title = if !app.search_query.is_empty() {
        format!(
            "Instances ({}/{})",
            app.filtered_instance_count(),
            app.total_instance_count()
        )
    } else {
        "Instances".to_string()
    };

    // Build column widths based on terminal width
    let widths = if width < 60 {
        vec![Constraint::Min(0)]
    } else if width < 80 {
        vec![Constraint::Min(20), Constraint::Length(14)]
    } else if width < 100 {
        vec![
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(12),
        ]
    } else {
        vec![
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(22),
        ]
    };

    let table = Table::new(rows, widths).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);

    // Register click regions for visible rows
    // Content starts at area.y + 1 (top border)
    for (row_idx, vrow) in visual.iter().enumerate() {
        let row_y = area.y + 1 + row_idx as u16;
        if row_y >= area.y + area.height.saturating_sub(1) {
            break; // past visible area (bottom border)
        }
        let row_rect = Rect {
            x: area.x,
            y: row_y,
            width: area.width,
            height: 1,
        };
        match vrow {
            VisualRow::GroupHeader { key, .. } => {
                app.register_click(row_rect, ClickAction::GroupHeader(key.clone()));
            }
            VisualRow::Instance(idx) => {
                app.register_click(row_rect, ClickAction::SelectItem(*idx));
            }
        }
    }

    // Scrollbar
    if let Some(sel) = selected_row {
        render_scrollbar(
            frame,
            area,
            total_visible,
            inner_height,
            sel.saturating_sub(inner_height / 2),
        );
    }
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
        let selected_running = app
            .selected_instance()
            .map(|i| app.is_instance_running(&i.id))
            .unwrap_or(false);

        let mut keys: Vec<(&str, &str, Option<Message>)> = vec![
            ("j/k", "Nav", None),
            ("l/Enter", "Launch", Some(Message::LaunchInstance)),
        ];
        if selected_running {
            keys.push(("x", "Kill", Some(Message::KillInstance)));
        }
        keys.extend_from_slice(&[
            ("/", "Search", Some(Message::StartSearch)),
            ("S", "Sort", Some(Message::CycleSortMode)),
            ("s", "Servers", Some(Message::OpenServerScreen)),
            ("a", "Account", Some(Message::OpenAccountScreen)),
            ("i", "Details", Some(Message::OpenInstanceDetails)),
            ("o", "Open", Some(Message::OpenInstanceFolder)),
            ("?", "Help", Some(Message::OpenHelp)),
            ("q", "Quit", Some(Message::Quit)),
        ]);
        render_footer_bar(app, frame, area, &keys);
    }
}
