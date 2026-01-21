use crate::app::{App, ClickAction};
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
            Constraint::Length(3), // Join on launch status
            Constraint::Min(0),    // Server list
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_join_status(app, frame, chunks[1]);
    render_server_list(app, frame, chunks[2]);
    render_footer(app, frame, chunks[3]);
}

fn render_header(app: &mut App, frame: &mut Frame, area: Rect) {
    let instance_name = app
        .selected_instance()
        .map(|i| i.name.as_str())
        .unwrap_or("Unknown");

    let back_text = "[Esc] Back";
    let back_x_offset = instance_name.len() + " - Servers".len() + 2;

    let header = Paragraph::new(Line::from(vec![
        Span::styled(instance_name, Style::default().fg(ui::PRIMARY).bold()),
        Span::styled(" - Servers", Style::default().fg(ui::PRIMARY)),
        Span::raw("  "),
        Span::styled(back_text, Style::default().fg(ui::MUTED)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);

    // Register back button click region
    let back_region = Rect {
        x: area.x + 1 + back_x_offset as u16,
        y: area.y,
        width: back_text.len() as u16,
        height: area.height,
    };
    app.register_click(back_region, ClickAction::GoBack);
}

fn render_join_status(app: &mut App, frame: &mut Frame, area: Rect) {
    let (enabled, address) = app
        .selected_instance()
        .and_then(|i| i.server_join.as_ref())
        .map(|sj| (sj.enabled, sj.address.as_str()))
        .unwrap_or((false, "None"));

    let checkbox = if enabled { "[x]" } else { "[ ]" };

    let status = Paragraph::new(Line::from(vec![
        Span::raw("Join on Launch: "),
        Span::styled(
            checkbox,
            if enabled {
                Style::default().fg(ui::ACTIVE)
            } else {
                Style::default().fg(ui::MUTED)
            },
        ),
        Span::raw(" "),
        Span::styled(
            address,
            if enabled {
                Style::default().fg(ui::ACTIVE)
            } else {
                Style::default().fg(ui::MUTED)
            },
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);

    // Register click region for the join checkbox area
    app.register_click(area, ClickAction::JoinCheckbox);
}

fn render_server_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;

    let join_address = app
        .selected_instance()
        .and_then(|i| i.server_join.as_ref())
        .filter(|sj| sj.enabled)
        .map(|sj| sj.address.as_str());

    let items: Vec<ListItem> = app
        .servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_selected = idx == app.selected_server_index;
            let is_join_server = join_address.map(|a| a == server.ip).unwrap_or(false);

            let prefix = if is_selected {
                SELECTED_PREFIX
            } else {
                UNSELECTED_PREFIX
            };
            let join_marker = if is_join_server { " [J]" } else { "" };

            let style = if is_selected {
                Style::default()
                    .fg(ui::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{:<20}", truncate(&server.name, 20)), style),
                Span::styled(&server.ip, Style::default().fg(ui::MUTED)),
                Span::styled(join_marker, Style::default().fg(ui::ACTIVE)),
            ]))
        })
        .collect();

    let total_items = items.len();

    let list = if items.is_empty() {
        List::new(vec![ListItem::new(Span::styled(
            "  No servers. Press 'a' to add one.",
            Style::default().fg(ui::MUTED),
        ))])
    } else {
        List::new(items)
    }
    .block(Block::default().borders(Borders::ALL).title("Servers"));

    frame.render_widget(list, area);

    // Register click regions for each visible server item
    for idx in 0..app.servers.len() {
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
        app.register_click(row_rect, ClickAction::SelectItem(idx));
    }

    // Scrollbar
    render_scrollbar(
        frame,
        area,
        total_items,
        inner_height,
        app.selected_server_index.saturating_sub(inner_height / 2),
    );
}

fn render_footer(app: &mut App, frame: &mut Frame, area: Rect) {
    let keys: &[(&str, &str, Option<Message>)] = &[
        ("j/k", "Nav", None),
        ("l/Enter", "Launch", Some(Message::LaunchWithServer)),
        ("J", "Join", Some(Message::SetJoinOnLaunch)),
        ("a", "Add", Some(Message::AddServer)),
        ("e", "Edit", Some(Message::EditServer)),
        ("d", "Del", Some(Message::DeleteServer)),
        ("h/Esc", "Back", Some(Message::Back)),
    ];
    render_footer_bar(app, frame, area, keys);
}
