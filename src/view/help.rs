use crate::app::{App, ClickAction};
use crate::theme::ui;
use crate::view::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

struct HelpEntry {
    key: &'static str,
    description: &'static str,
}

struct HelpSection {
    title: &'static str,
    entries: &'static [HelpEntry],
}

const NAVIGATION: &[HelpEntry] = &[
    HelpEntry {
        key: "j/k / ↑/↓",
        description: "Move down/up",
    },
    HelpEntry {
        key: "g/G / Home/End",
        description: "Go to top/bottom",
    },
    HelpEntry {
        key: "l/Enter",
        description: "Select/Launch",
    },
    HelpEntry {
        key: "h/Esc",
        description: "Back",
    },
    HelpEntry {
        key: "Ctrl+j/k",
        description: "Jump to next/prev group",
    },
];

const INSTANCE_KEYS: &[HelpEntry] = &[
    HelpEntry {
        key: "s",
        description: "Open servers",
    },
    HelpEntry {
        key: "a",
        description: "Select account",
    },
    HelpEntry {
        key: "i",
        description: "Instance details",
    },
    HelpEntry {
        key: "o",
        description: "Open folder",
    },
    HelpEntry {
        key: "S",
        description: "Cycle sort mode",
    },
    HelpEntry {
        key: "L",
        description: "Instance logs",
    },
    HelpEntry {
        key: "gl",
        description: "Launcher logs",
    },
    HelpEntry {
        key: "Tab",
        description: "Collapse/expand group",
    },
    HelpEntry {
        key: "x",
        description: "Kill running instance",
    },
    HelpEntry {
        key: "/",
        description: "Start search",
    },
];

const SERVER_KEYS: &[HelpEntry] = &[
    HelpEntry {
        key: "a",
        description: "Add server",
    },
    HelpEntry {
        key: "e",
        description: "Edit server",
    },
    HelpEntry {
        key: "d",
        description: "Delete server",
    },
    HelpEntry {
        key: "J",
        description: "Set join-on-launch",
    },
];

const LOG_KEYS: &[HelpEntry] = &[
    HelpEntry {
        key: "J/K / PgUp/Dn",
        description: "Scroll content",
    },
    HelpEntry {
        key: "/",
        description: "Search log content",
    },
    HelpEntry {
        key: "n/N",
        description: "Next/prev match",
    },
    HelpEntry {
        key: "1-4",
        description: "Filter: ERR/WARN/INFO/DEBUG",
    },
    HelpEntry {
        key: "0",
        description: "Show all levels",
    },
    HelpEntry {
        key: "e",
        description: "Open in editor",
    },
    HelpEntry {
        key: "o",
        description: "Open folder",
    },
];

const GLOBAL_KEYS: &[HelpEntry] = &[
    HelpEntry {
        key: "?",
        description: "Show/hide this help",
    },
    HelpEntry {
        key: "q",
        description: "Quit",
    },
];

const HELP_SECTIONS: &[HelpSection] = &[
    HelpSection {
        title: "Navigation",
        entries: NAVIGATION,
    },
    HelpSection {
        title: "Instance List",
        entries: INSTANCE_KEYS,
    },
    HelpSection {
        title: "Server List",
        entries: SERVER_KEYS,
    },
    HelpSection {
        title: "Log Viewer",
        entries: LOG_KEYS,
    },
    HelpSection {
        title: "Global",
        entries: GLOBAL_KEYS,
    },
];

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let help_width = 55.min(area.width.saturating_sub(4));
    let help_height = 40.min(area.height.saturating_sub(4));

    let help_area = centered_rect(help_width, help_height, area);

    // Click outside help dismisses, click inside absorbs
    app.register_click(area, ClickAction::DismissOverlay);
    app.register_click(help_area, ClickAction::Noop);

    frame.render_widget(Clear, help_area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " Keybindings",
        Style::default().fg(ui::PRIMARY).bold(),
    )));
    lines.push(Line::from(""));

    for section in HELP_SECTIONS {
        lines.push(Line::from(Span::styled(
            format!(" {}", section.title),
            Style::default()
                .fg(ui::HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        )));

        for entry in section.entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<16}", entry.key),
                    Style::default().fg(ui::ACTIVE),
                ),
                Span::styled(entry.description, Style::default().fg(ui::TEXT)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Calculate total lines for scrolling
    let inner_height = help_height.saturating_sub(2) as usize;
    let total_lines = lines.len();
    let scroll_offset = app
        .help_scroll_offset
        .min(total_lines.saturating_sub(inner_height));

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(inner_height)
        .collect();

    let title = if total_lines > inner_height {
        format!(
            "Help (j/k to scroll, {}/{})",
            scroll_offset + 1,
            total_lines
        )
    } else {
        "Help".to_string()
    };

    let help = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(ui::HELP_BORDER)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, help_area);
}
