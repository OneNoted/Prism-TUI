use crate::app::App;
use crate::message::Message;
use crate::theme::ui;
use crate::view::render_footer_bar;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    render_details(app, frame, chunks[0]);
    render_footer(app, frame, chunks[1]);
}

fn render_details(app: &mut App, frame: &mut Frame, area: Rect) {
    let instance = match app.selected_instance() {
        Some(i) => i,
        None => {
            let empty = Paragraph::new("No instance selected")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Instance Details"),
                )
                .style(Style::default().fg(ui::MUTED));
            frame.render_widget(empty, area);
            return;
        }
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  Name:           ", Style::default().fg(ui::MUTED)),
            Span::styled(&instance.name, Style::default().fg(ui::TEXT).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Path:           ", Style::default().fg(ui::MUTED)),
            Span::styled(
                instance.path.display().to_string(),
                Style::default().fg(ui::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Version:        ", Style::default().fg(ui::MUTED)),
            Span::styled(&instance.minecraft_version, Style::default().fg(ui::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  Mod Loader:     ", Style::default().fg(ui::MUTED)),
            Span::styled(
                instance.mod_loader.as_deref().unwrap_or("None"),
                Style::default().fg(ui::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Group:          ", Style::default().fg(ui::MUTED)),
            Span::styled(
                instance.group.as_deref().unwrap_or("Ungrouped"),
                Style::default().fg(ui::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Playtime:       ", Style::default().fg(ui::MUTED)),
            Span::styled(
                instance.formatted_playtime_full(),
                Style::default().fg(ui::ACTIVE),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Last Launch:    ", Style::default().fg(ui::MUTED)),
            Span::styled(
                instance.formatted_last_launch(),
                Style::default().fg(ui::TEXT),
            ),
        ]),
        Line::from(""),
    ];

    // Server Join
    let join_text = instance
        .server_join
        .as_ref()
        .map(|sj| {
            if sj.enabled {
                format!("Enabled ({})", sj.address)
            } else {
                format!("Disabled ({})", sj.address)
            }
        })
        .unwrap_or_else(|| "Not configured".to_string());

    lines.push(Line::from(vec![
        Span::styled("  Join on Launch: ", Style::default().fg(ui::MUTED)),
        Span::styled(join_text, Style::default().fg(ui::TEXT)),
    ]));

    lines.push(Line::from(""));

    // Counts
    let mods = instance.mods_count();
    let saves = instance.saves_count();
    let packs = instance.resource_packs_count();

    lines.push(Line::from(vec![
        Span::styled("  Mods:           ", Style::default().fg(ui::MUTED)),
        Span::styled(format!("{}", mods), Style::default().fg(ui::TEXT)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Saves:          ", Style::default().fg(ui::MUTED)),
        Span::styled(format!("{}", saves), Style::default().fg(ui::TEXT)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Resource Packs: ", Style::default().fg(ui::MUTED)),
        Span::styled(format!("{}", packs), Style::default().fg(ui::TEXT)),
    ]));

    let title = format!("Instance Details: {}", instance.name);
    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(details, area);
}

fn render_footer(app: &mut App, frame: &mut Frame, area: Rect) {
    let keys: &[(&str, &str, Option<Message>)] = &[
        ("h/Esc", "Back", Some(Message::Back)),
        ("o", "Open Folder", Some(Message::OpenInstanceFolder)),
        ("q", "Quit", Some(Message::Quit)),
    ];
    render_footer_bar(app, frame, area, keys);
}
