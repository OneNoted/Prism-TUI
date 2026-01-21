mod accounts;
mod details;
mod help;
mod instances;
mod logs;
mod servers;

use crate::app::{App, ClickAction, InputMode, Screen};
use crate::message::Message;
use crate::theme::ui;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs,
};

pub(crate) const SELECTED_PREFIX: &str = " > ";
pub(crate) const UNSELECTED_PREFIX: &str = "   ";

pub fn render(app: &mut App, frame: &mut Frame) {
    app.click_regions.clear();
    let area = frame.area();

    // Split into tab bar + content
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_tab_bar(app, frame, outer[0]);
    let content_area = outer[1];

    match app.screen {
        Screen::Instances => instances::render(app, frame, content_area),
        Screen::Accounts => accounts::render(app, frame, content_area),
        Screen::Servers => servers::render(app, frame, content_area),
        Screen::Logs => logs::render(app, frame, content_area),
        Screen::InstanceDetails => details::render(app, frame, content_area),
        Screen::Help => {
            instances::render(app, frame, content_area);
            help::render(app, frame, content_area);
        }
    }

    // Render input dialog overlay (but not for search or log search, which are rendered inline)
    if app.input_mode != InputMode::Normal
        && app.input_mode != InputMode::Search
        && app.input_mode != InputMode::LogSearch
    {
        render_input_dialog(app, frame, area);
    }

    // Render error message if present
    if let Some(ref error) = app.error_message {
        let error = error.clone();
        render_error(&error, app, frame, area);
    }
}

fn render_tab_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    let titles = vec!["Instances", "Accounts", "Servers", "Logs"];
    let selected = match app.screen {
        Screen::Instances | Screen::InstanceDetails | Screen::Help => 0,
        Screen::Accounts => 1,
        Screen::Servers => 2,
        Screen::Logs => 3,
    };

    let tabs = Tabs::new(titles.clone())
        .select(selected)
        .style(Style::default().fg(ui::MUTED))
        .highlight_style(Style::default().fg(ui::PRIMARY).bold())
        .divider(" | ");

    frame.render_widget(tabs, area);

    // Register click regions for each tab
    // Tabs widget renders: " Title0 | Title1 | Title2 | Title3 "
    // Each title is preceded by a space and followed by divider " | " (3 chars), except the last
    let mut x = area.x + 1; // initial padding
    for (i, title) in titles.iter().enumerate() {
        let title_width = title.len() as u16;
        let region = Rect {
            x,
            y: area.y,
            width: title_width,
            height: 1,
        };
        app.register_click(region, ClickAction::SwitchTab(i));
        x += title_width;
        if i < titles.len() - 1 {
            x += 3; // " | " divider
        }
    }
}

fn render_input_dialog(app: &mut App, frame: &mut Frame, area: Rect) {
    let dialog_width = 50.min(area.width.saturating_sub(4));
    let dialog_height = 5;

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Register dismiss overlay for outside clicks first, then noop for dialog body
    app.register_click(area, ClickAction::DismissOverlay);
    app.register_click(dialog_area, ClickAction::Noop);

    frame.render_widget(Clear, dialog_area);

    let (title, prompt) = match app.input_mode {
        InputMode::AddServerName => ("Add Server", "Server name:"),
        InputMode::AddServerAddress => ("Add Server", "Server address:"),
        InputMode::EditServerName => ("Edit Server", "Server name:"),
        InputMode::EditServerAddress => ("Edit Server", "Server address:"),
        InputMode::ConfirmDelete => ("Confirm Delete", "Delete this server? (y/n)"),
        InputMode::Normal | InputMode::Search | InputMode::LogSearch => return,
    };

    let content = if app.input_mode == InputMode::ConfirmDelete {
        prompt.to_string()
    } else {
        format!("{} {}_", prompt, app.input_buffer)
    };

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(ui::DIALOG_BORDER)),
        )
        .style(Style::default().fg(ui::TEXT));

    frame.render_widget(dialog, dialog_area);
}

fn render_error(error: &str, app: &mut App, frame: &mut Frame, area: Rect) {
    let error_width = (error.len() as u16 + 4).min(area.width.saturating_sub(4));
    let error_height = 3;

    let error_area = Rect {
        x: area.x + (area.width.saturating_sub(error_width)) / 2,
        y: area.height.saturating_sub(error_height + 2),
        width: error_width,
        height: error_height,
    };

    // Click outside error dismisses it, click inside absorbs
    app.register_click(area, ClickAction::DismissOverlay);
    app.register_click(error_area, ClickAction::Noop);

    frame.render_widget(Clear, error_area);

    let error_widget = Paragraph::new(error)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .border_style(Style::default().fg(ui::ERROR)),
        )
        .style(Style::default().fg(ui::ERROR));

    frame.render_widget(error_widget, error_area);
}

pub(super) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub(crate) fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    total_items: usize,
    visible_items: usize,
    offset: usize,
) {
    if total_items > visible_items {
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        let mut scrollbar_state =
            ScrollbarState::new(total_items.saturating_sub(visible_items)).position(offset);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

pub(crate) fn render_footer_bar(
    app: &mut App,
    frame: &mut Frame,
    area: Rect,
    keys: &[(&str, &str, Option<Message>)],
) {
    let mut spans = Vec::new();

    // inner_x tracks x position inside the block (border = 1 col each side)
    let mut inner_x: u16 = 0;

    for (i, (key, action, msg)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(ui::MUTED)));
            inner_x += 2;
        }
        let key_len = key.len() as u16;
        let action_text = format!(" {}", action);
        let action_len = action_text.len() as u16;
        let total_len = key_len + action_len;

        spans.push(Span::styled(*key, Style::default().fg(ui::HIGHLIGHT)));
        spans.push(Span::styled(action_text, Style::default().fg(ui::MUTED)));

        if let Some(m) = msg {
            // Register click region: area.x + 1 (left border) + inner_x
            let region = Rect {
                x: area.x + 1 + inner_x,
                y: area.y,
                width: total_len,
                height: area.height,
            };
            app.register_click(region, ClickAction::FooterAction(m.clone()));
        }

        inner_x += total_len;
    }

    let footer = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
