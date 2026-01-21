use crate::actions::{launch_instance, open_folder, open_in_editor};
use crate::app::{App, ClickAction, InputMode, LogLevel, LogSource, RunningInstance, Screen};
use crate::data::{Instance, Server, load_log_content, load_log_entries};
use crate::message::Message;
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn update(app: &mut App, msg: Message) {
    // Clear error on any input except Tick
    if !matches!(msg, Message::Tick) {
        app.clear_error();
    }

    match msg {
        Message::Key(key) => handle_key(app, key.code, key.modifiers),
        Message::Mouse(mouse) => handle_mouse(app, mouse),
        Message::Tick => {
            if !app.running_instances.is_empty()
                && app.last_process_scan.elapsed() >= Duration::from_secs(2)
            {
                app.last_process_scan = Instant::now();
                poll_running_instances(app);
            }
        }

        Message::SwitchToScreen(screen) => match screen {
            Screen::Instances => {
                app.screen = Screen::Instances;
            }
            Screen::Accounts => {
                update(app, Message::OpenAccountScreen);
            }
            Screen::Servers => {
                update(app, Message::OpenServerScreen);
            }
            Screen::Logs => {
                update(app, Message::OpenInstanceLogs);
            }
            _ => {}
        },

        Message::SelectInstance(idx) => {
            if idx < app.visible_instance_count() {
                app.selected_instance_index = idx;
                app.selected_group_index = app.group_index_for_instance(idx);
            }
        }

        Message::LaunchInstance => {
            if let Some(instance) = app.selected_instance() {
                let instance_id = instance.id.clone();
                if app.is_instance_running(&instance_id) {
                    app.set_error("Instance is already running".into());
                    return;
                }
                let server = instance
                    .server_join
                    .as_ref()
                    .filter(|sj| sj.enabled)
                    .map(|sj| sj.address.clone());
                let account = app.active_account.as_ref().map(|a| a.username.clone());

                if let Err(e) = launch_instance(&instance_id, account.as_deref(), server.as_deref())
                {
                    app.set_error(format!("Launch failed: {}", e));
                } else {
                    app.running_instances.insert(
                        instance_id,
                        RunningInstance {
                            pid: None,
                            launched_at: Instant::now(),
                        },
                    );
                }
            }
        }

        Message::KillInstance => {
            if let Some(instance) = app.selected_instance() {
                let id = instance.id.clone();
                if let Some(running) = app.running_instances.remove(&id)
                    && let Some(pid) = running.pid
                    && let Some(process) = app.system.process(pid)
                {
                    let killed = process.kill_with(sysinfo::Signal::Term).unwrap_or(false);
                    if !killed {
                        process.kill();
                    }
                }
            }
        }

        Message::OpenInstanceFolder => {
            if let Some(instance) = app.selected_instance()
                && let Err(e) = open_folder(&instance.path)
            {
                app.set_error(format!("Failed to open folder: {}", e));
            }
        }

        Message::OpenInstanceDetails => {
            if app.selected_instance().is_some() {
                app.previous_screen = Some(app.screen);
                app.screen = Screen::InstanceDetails;
            }
        }

        Message::SelectAccount(idx) => {
            if idx < app.accounts.len() {
                app.selected_account_index = idx;
            }
        }

        Message::ConfirmAccountSelection => {
            if let Some(account) = app.selected_account().cloned() {
                app.active_account = Some(account);
                app.screen = Screen::Instances;
            }
        }

        Message::SelectServer(idx) => {
            if idx < app.servers.len() {
                app.selected_server_index = idx;
            }
        }

        Message::AddServer => {
            app.input_mode = InputMode::AddServerName;
            app.input_buffer.clear();
            app.edit_server_name.clear();
            app.edit_server_address.clear();
        }

        Message::EditServer => {
            if let Some(server) = app.selected_server().cloned() {
                app.edit_server_name = server.name.clone();
                app.edit_server_address = server.ip.clone();
                app.input_buffer = server.name;
                app.input_mode = InputMode::EditServerName;
            }
        }

        Message::DeleteServer => {
            if !app.servers.is_empty() {
                app.input_mode = InputMode::ConfirmDelete;
            }
        }

        Message::ConfirmDeleteServer => {
            if app.selected_server_index < app.servers.len() {
                app.servers.remove(app.selected_server_index);
                if app.servers.is_empty() {
                    app.selected_server_index = 0;
                } else if app.selected_server_index >= app.servers.len() {
                    app.selected_server_index = app.servers.len() - 1;
                }
                if let Err(e) = app.save_servers_for_instance() {
                    app.set_error(format!("Failed to save servers: {}", e));
                }
            }
            app.input_mode = InputMode::Normal;
        }

        Message::SetJoinOnLaunch => {
            if let Some(server) = app.selected_server().cloned()
                && let Some(instance) = app.selected_instance_mut()
            {
                let currently_set = instance
                    .server_join
                    .as_ref()
                    .map(|sj| sj.enabled && sj.address == server.ip)
                    .unwrap_or(false);

                if currently_set {
                    if let Err(e) = instance.set_server_join(false, Some(server.ip)) {
                        app.set_error(format!("Failed to update config: {}", e));
                    }
                } else if let Err(e) = instance.set_server_join(true, Some(server.ip)) {
                    app.set_error(format!("Failed to update config: {}", e));
                }
            }
        }

        Message::LaunchWithServer => {
            if let (Some(instance), Some(server)) = (app.selected_instance(), app.selected_server())
            {
                let instance_id = instance.id.clone();
                if app.is_instance_running(&instance_id) {
                    app.set_error("Instance is already running".into());
                    return;
                }
                let server_addr = server.ip.clone();
                let account = app.active_account.as_ref().map(|a| a.username.clone());

                if let Err(e) =
                    launch_instance(&instance_id, account.as_deref(), Some(&server_addr))
                {
                    app.set_error(format!("Launch failed: {}", e));
                } else {
                    app.running_instances.insert(
                        instance_id,
                        RunningInstance {
                            pid: None,
                            launched_at: Instant::now(),
                        },
                    );
                }
            }
        }

        Message::InputChar(c) => {
            app.input_buffer.push(c);
        }

        Message::InputBackspace => {
            app.input_buffer.pop();
        }

        Message::InputConfirm => match app.input_mode {
            InputMode::AddServerName => {
                let name = app.input_buffer.trim().to_string();
                if name.is_empty() {
                    app.set_error("Server name cannot be empty".to_string());
                } else {
                    app.edit_server_name = name;
                    app.input_buffer.clear();
                    app.input_mode = InputMode::AddServerAddress;
                }
            }
            InputMode::AddServerAddress => {
                let address = app.input_buffer.trim().to_string();
                if let Err(e) = validate_server_address(&address) {
                    app.set_error(e);
                } else {
                    app.edit_server_address = address;
                    app.servers.push(Server {
                        name: app.edit_server_name.clone(),
                        ip: app.edit_server_address.clone(),
                    });
                    if let Err(e) = app.save_servers_for_instance() {
                        app.set_error(format!("Failed to save servers: {}", e));
                    }
                    app.input_buffer.clear();
                    app.input_mode = InputMode::Normal;
                }
            }
            InputMode::EditServerName => {
                let name = app.input_buffer.trim().to_string();
                if name.is_empty() {
                    app.set_error("Server name cannot be empty".to_string());
                } else {
                    app.edit_server_name = name;
                    app.input_buffer = app.edit_server_address.clone();
                    app.input_mode = InputMode::EditServerAddress;
                }
            }
            InputMode::EditServerAddress => {
                let address = app.input_buffer.trim().to_string();
                if let Err(e) = validate_server_address(&address) {
                    app.set_error(e);
                } else {
                    app.edit_server_address = address;
                    if let Some(server) = app.servers.get_mut(app.selected_server_index) {
                        server.name = app.edit_server_name.clone();
                        server.ip = app.edit_server_address.clone();
                        if let Err(e) = app.save_servers_for_instance() {
                            app.set_error(format!("Failed to save servers: {}", e));
                        }
                    }
                    app.input_buffer.clear();
                    app.input_mode = InputMode::Normal;
                }
            }
            _ => {}
        },

        Message::InputCancel => {
            app.input_buffer.clear();
            app.input_mode = InputMode::Normal;
        }

        Message::OpenAccountScreen => {
            app.previous_screen = Some(app.screen);
            app.screen = Screen::Accounts;
        }

        Message::OpenServerScreen => {
            if app.selected_instance().is_some() {
                if let Err(e) = app.load_servers_for_instance() {
                    app.set_error(format!("Failed to load servers: {}", e));
                } else {
                    app.previous_screen = Some(app.screen);
                    app.screen = Screen::Servers;
                }
            }
        }

        Message::OpenHelp => {
            app.previous_screen = Some(app.screen);
            app.help_scroll_offset = 0;
            app.screen = Screen::Help;
        }

        Message::Back => {
            if let Some(prev) = app.previous_screen.take() {
                app.screen = prev;
            } else {
                app.screen = Screen::Instances;
            }
        }

        Message::OpenInstanceLogs => {
            if let Some(instance) = app.selected_instance() {
                let logs_dir = instance.logs_dir();
                match load_log_entries(&logs_dir) {
                    Ok(entries) => {
                        app.log_entries = entries;
                        app.selected_log_index = 0;
                        app.log_content.clear();
                        app.log_scroll_offset = 0;
                        app.log_source = LogSource::Instance;
                        app.log_search_query.clear();
                        app.log_search_matches.clear();
                        app.log_level_filter.clear();
                        app.previous_screen = Some(app.screen);
                        app.screen = Screen::Logs;
                    }
                    Err(e) => {
                        app.set_error(format!("Failed to load logs: {}", e));
                    }
                }
            }
        }

        Message::OpenLauncherLogs => {
            let logs_dir = app.data_dir.join("logs");
            match load_log_entries(&logs_dir) {
                Ok(entries) => {
                    app.log_entries = entries;
                    app.selected_log_index = 0;
                    app.log_content.clear();
                    app.log_scroll_offset = 0;
                    app.log_source = LogSource::Launcher;
                    app.log_search_query.clear();
                    app.log_search_matches.clear();
                    app.log_level_filter.clear();
                    app.previous_screen = Some(app.screen);
                    app.screen = Screen::Logs;
                }
                Err(e) => {
                    app.set_error(format!("Failed to load logs: {}", e));
                }
            }
        }

        Message::SelectLog(idx) => {
            if idx < app.log_entries.len() {
                app.selected_log_index = idx;
                app.log_content.clear();
                app.log_scroll_offset = 0;
            }
        }

        Message::LoadLogContent => {
            if let Some(entry) = app.log_entries.get(app.selected_log_index) {
                match load_log_content(&entry.path) {
                    Ok(content) => {
                        app.log_content = content;
                        app.log_scroll_offset = 0;
                        // Re-run search if active
                        if !app.log_search_query.is_empty() {
                            app.update_log_search();
                        }
                    }
                    Err(e) => {
                        app.set_error(format!("Failed to load log content: {}", e));
                    }
                }
            }
        }

        Message::ScrollLogUp(amount) => {
            app.log_scroll_offset = app.log_scroll_offset.saturating_sub(amount);
        }

        Message::ScrollLogDown(amount) => {
            let max_offset = app.filtered_log_content().len().saturating_sub(1);
            app.log_scroll_offset = (app.log_scroll_offset + amount).min(max_offset);
        }

        Message::OpenLogInEditor => {
            if let Some(entry) = app.log_entries.get(app.selected_log_index)
                && let Err(e) = open_in_editor(&entry.path)
            {
                app.set_error(format!("Failed to open editor: {}", e));
            }
        }

        Message::OpenLogFolder => {
            if let Some(entry) = app.log_entries.get(app.selected_log_index)
                && let Some(parent) = entry.path.parent()
                && let Err(e) = open_folder(parent)
            {
                app.set_error(format!("Failed to open folder: {}", e));
            }
        }

        // Log search
        Message::StartLogSearch => {
            app.input_mode = InputMode::LogSearch;
            app.log_search_query.clear();
            app.log_search_matches.clear();
            app.log_search_current = 0;
        }

        Message::LogSearchChar(c) => {
            app.log_search_query.push(c);
            app.update_log_search();
        }

        Message::LogSearchBackspace => {
            app.log_search_query.pop();
            app.update_log_search();
        }

        Message::LogSearchConfirm => {
            app.input_mode = InputMode::Normal;
        }

        Message::LogSearchCancel => {
            app.log_search_query.clear();
            app.log_search_matches.clear();
            app.log_search_current = 0;
            app.input_mode = InputMode::Normal;
        }

        Message::LogSearchNext => {
            app.log_search_next();
        }

        Message::LogSearchPrev => {
            app.log_search_prev();
        }

        // Log level filtering
        Message::ToggleLogLevel(level) => {
            if app.log_level_filter.contains(&level) {
                app.log_level_filter.remove(&level);
            } else {
                app.log_level_filter.insert(level);
            }
        }

        Message::ShowAllLogLevels => {
            app.log_level_filter.clear();
        }

        // Search
        Message::StartSearch => {
            app.input_mode = InputMode::Search;
            app.input_buffer.clear();
        }

        Message::SearchChar(c) => {
            app.input_buffer.push(c);
            app.update_search(app.input_buffer.clone());
        }

        Message::SearchBackspace => {
            app.input_buffer.pop();
            app.update_search(app.input_buffer.clone());
        }

        Message::SearchConfirm => {
            app.input_mode = InputMode::Normal;
        }

        Message::SearchCancel => {
            app.input_buffer.clear();
            app.clear_search();
            app.input_mode = InputMode::Normal;
        }

        // Sorting
        Message::CycleSortMode => {
            app.sort_mode = app.sort_mode.next();
            app.sort_and_group_instances();
            app.selected_instance_index = 0;
            app.selected_group_index = app.group_index_for_instance(0);
            app.save_config();
        }

        Message::ToggleSortDirection => {
            app.sort_ascending = !app.sort_ascending;
            app.sort_and_group_instances();
            app.selected_instance_index = 0;
            app.selected_group_index = app.group_index_for_instance(0);
            app.save_config();
        }

        // Collapsible groups
        Message::ToggleGroupCollapse => {
            if let Some(key) = app.selected_group_key() {
                toggle_group_collapse(app, &key);
            }
        }

        Message::NextGroup => {
            let count = app.grouped_instances.len();
            if count > 0 {
                app.selected_group_index = (app.selected_group_index + 1) % count;
                if let Some(first) = app.first_instance_in_group(app.selected_group_index) {
                    app.selected_instance_index = first;
                }
            }
        }

        Message::PrevGroup => {
            let count = app.grouped_instances.len();
            if count > 0 {
                if app.selected_group_index == 0 {
                    app.selected_group_index = count - 1;
                } else {
                    app.selected_group_index -= 1;
                }
                if let Some(first) = app.first_instance_in_group(app.selected_group_index) {
                    app.selected_instance_index = first;
                }
            }
        }

        // Help scrolling
        Message::ScrollHelpUp => {
            app.help_scroll_offset = app.help_scroll_offset.saturating_sub(1);
        }

        Message::ScrollHelpDown => {
            app.help_scroll_offset += 1;
        }

        Message::Quit => {
            app.running = false;
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Handle input modes
    if app.input_mode != InputMode::Normal {
        match app.input_mode {
            InputMode::Search => match code {
                KeyCode::Char(c) => update(app, Message::SearchChar(c)),
                KeyCode::Backspace => update(app, Message::SearchBackspace),
                KeyCode::Enter => update(app, Message::SearchConfirm),
                KeyCode::Esc => update(app, Message::SearchCancel),
                _ => {}
            },
            InputMode::LogSearch => match code {
                KeyCode::Char(c) => update(app, Message::LogSearchChar(c)),
                KeyCode::Backspace => update(app, Message::LogSearchBackspace),
                KeyCode::Enter => update(app, Message::LogSearchConfirm),
                KeyCode::Esc => update(app, Message::LogSearchCancel),
                _ => {}
            },
            InputMode::ConfirmDelete => match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    update(app, Message::ConfirmDeleteServer);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    update(app, Message::InputCancel);
                }
                _ => {}
            },
            _ => match code {
                KeyCode::Char(c) => update(app, Message::InputChar(c)),
                KeyCode::Backspace => update(app, Message::InputBackspace),
                KeyCode::Enter => update(app, Message::InputConfirm),
                KeyCode::Esc => update(app, Message::InputCancel),
                _ => {}
            },
        }
        return;
    }

    // Normal mode keybindings
    match app.screen {
        Screen::Instances => handle_instances_key(app, code, modifiers),
        Screen::Accounts => handle_accounts_key(app, code),
        Screen::Servers => handle_servers_key(app, code),
        Screen::Logs => handle_logs_key(app, code),
        Screen::InstanceDetails => handle_details_key(app, code),
        Screen::Help => handle_help_key(app, code),
    }
}

fn rect_contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Double-click detection
            let now = Instant::now();
            let is_double_click = app
                .last_click_time
                .map(|t| now.duration_since(t) < Duration::from_millis(400))
                .unwrap_or(false)
                && app.last_click_pos == (col, row);
            app.last_click_time = Some(now);
            app.last_click_pos = (col, row);

            // Find matching click region (reverse for z-order: last registered wins)
            let target = app
                .click_regions
                .iter()
                .rev()
                .find(|r| rect_contains(r.rect, col, row))
                .map(|r| r.action.clone());

            match target {
                Some(ClickAction::SwitchTab(i)) => {
                    let screen = match i {
                        0 => Screen::Instances,
                        1 => Screen::Accounts,
                        2 => Screen::Servers,
                        3 => Screen::Logs,
                        _ => return,
                    };
                    update(app, Message::SwitchToScreen(screen));
                }
                Some(ClickAction::SelectItem(idx)) => match app.screen {
                    Screen::Instances => {
                        update(app, Message::SelectInstance(idx));
                        if is_double_click {
                            update(app, Message::LaunchInstance);
                        }
                    }
                    Screen::Accounts => {
                        update(app, Message::SelectAccount(idx));
                        if is_double_click {
                            update(app, Message::ConfirmAccountSelection);
                        }
                    }
                    Screen::Servers => {
                        update(app, Message::SelectServer(idx));
                        if is_double_click {
                            update(app, Message::LaunchWithServer);
                        }
                    }
                    _ => {}
                },
                Some(ClickAction::GroupHeader(key)) => {
                    toggle_group_collapse(app, &key);
                }
                Some(ClickAction::FooterAction(msg)) => {
                    update(app, msg);
                }
                Some(ClickAction::JoinCheckbox) => {
                    update(app, Message::SetJoinOnLaunch);
                }
                Some(ClickAction::GoBack) => {
                    update(app, Message::Back);
                }
                Some(ClickAction::DismissOverlay) => match app.screen {
                    Screen::Help => {
                        update(app, Message::Back);
                    }
                    _ => {
                        if app.error_message.is_some() {
                            app.clear_error();
                        } else if app.input_mode != InputMode::Normal {
                            update(app, Message::InputCancel);
                        }
                    }
                },
                Some(ClickAction::SelectLogFile(idx)) => {
                    update(app, Message::SelectLog(idx));
                    if is_double_click {
                        update(app, Message::LoadLogContent);
                    }
                }
                Some(ClickAction::ScrollLogPreview) | Some(ClickAction::Noop) => {}
                None => {}
            }
        }
        MouseEventKind::ScrollUp => {
            // Check if scrolling over log preview area
            if app.screen == Screen::Logs {
                let over_preview = app
                    .click_regions
                    .iter()
                    .rev()
                    .find(|r| rect_contains(r.rect, col, row))
                    .map(|r| matches!(r.action, ClickAction::ScrollLogPreview))
                    .unwrap_or(false);
                if over_preview {
                    update(app, Message::ScrollLogUp(3));
                    return;
                }
                let over_file_list = app
                    .click_regions
                    .iter()
                    .rev()
                    .find(|r| rect_contains(r.rect, col, row))
                    .map(|r| matches!(r.action, ClickAction::SelectLogFile(_)))
                    .unwrap_or(false);
                if over_file_list && app.selected_log_index > 0 {
                    update(app, Message::SelectLog(app.selected_log_index - 1));
                    return;
                }
            }
            match app.screen {
                Screen::Instances => {
                    let prev_idx = app
                        .filtered_instance_indices
                        .iter()
                        .position(|&idx| idx == app.selected_instance_index)
                        .filter(|&pos| pos > 0)
                        .and_then(|pos| app.filtered_instance_indices.get(pos - 1).copied());
                    if let Some(idx) = prev_idx {
                        update(app, Message::SelectInstance(idx));
                    }
                }
                Screen::Accounts => {
                    let prev_idx = app
                        .filtered_account_indices
                        .iter()
                        .position(|&idx| idx == app.selected_account_index)
                        .filter(|&pos| pos > 0)
                        .and_then(|pos| app.filtered_account_indices.get(pos - 1).copied());
                    if let Some(idx) = prev_idx {
                        update(app, Message::SelectAccount(idx));
                    }
                }
                Screen::Servers => {
                    if app.selected_server_index > 0 {
                        update(app, Message::SelectServer(app.selected_server_index - 1));
                    }
                }
                Screen::Logs => {
                    // Fallback: scroll log content if loaded, else navigate file list
                    if !app.log_content.is_empty() {
                        update(app, Message::ScrollLogUp(3));
                    } else if app.selected_log_index > 0 {
                        update(app, Message::SelectLog(app.selected_log_index - 1));
                    }
                }
                Screen::Help => {
                    update(app, Message::ScrollHelpUp);
                }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            // Check if scrolling over log preview area
            if app.screen == Screen::Logs {
                let over_preview = app
                    .click_regions
                    .iter()
                    .rev()
                    .find(|r| rect_contains(r.rect, col, row))
                    .map(|r| matches!(r.action, ClickAction::ScrollLogPreview))
                    .unwrap_or(false);
                if over_preview {
                    update(app, Message::ScrollLogDown(3));
                    return;
                }
                let over_file_list = app
                    .click_regions
                    .iter()
                    .rev()
                    .find(|r| rect_contains(r.rect, col, row))
                    .map(|r| matches!(r.action, ClickAction::SelectLogFile(_)))
                    .unwrap_or(false);
                if over_file_list && app.selected_log_index + 1 < app.log_entries.len() {
                    update(app, Message::SelectLog(app.selected_log_index + 1));
                    return;
                }
            }
            match app.screen {
                Screen::Instances => {
                    let next_idx = app
                        .filtered_instance_indices
                        .iter()
                        .position(|&idx| idx == app.selected_instance_index)
                        .and_then(|pos| app.filtered_instance_indices.get(pos + 1).copied());
                    if let Some(idx) = next_idx {
                        update(app, Message::SelectInstance(idx));
                    }
                }
                Screen::Accounts => {
                    let next_idx = app
                        .filtered_account_indices
                        .iter()
                        .position(|&idx| idx == app.selected_account_index)
                        .and_then(|pos| app.filtered_account_indices.get(pos + 1).copied());
                    if let Some(idx) = next_idx {
                        update(app, Message::SelectAccount(idx));
                    }
                }
                Screen::Servers => {
                    if app.selected_server_index + 1 < app.servers.len() {
                        update(app, Message::SelectServer(app.selected_server_index + 1));
                    }
                }
                Screen::Logs => {
                    if !app.log_content.is_empty() {
                        update(app, Message::ScrollLogDown(3));
                    } else if app.selected_log_index + 1 < app.log_entries.len() {
                        update(app, Message::SelectLog(app.selected_log_index + 1));
                    }
                }
                Screen::Help => {
                    update(app, Message::ScrollHelpDown);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_instances_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Ctrl+j/k/Up/Down for group navigation
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                update(app, Message::NextGroup);
                return;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                update(app, Message::PrevGroup);
                return;
            }
            _ => {}
        }
    }

    // Handle 2-key combo: g followed by l opens launcher logs
    if let Some(pending) = app.pending_key {
        app.pending_key = None;
        if pending == 'g' && code == KeyCode::Char('l') {
            update(app, Message::OpenLauncherLogs);
            return;
        }
        // If it was 'g' followed by something else, handle 'g' as go-to-top
        if pending == 'g'
            && let Some(first) = app.filtered_instance_indices.first().copied()
        {
            update(app, Message::SelectInstance(first));
        }
        // Don't return - process this key too if it's not 'l'
    }

    // Helper to find current position in filtered list
    let find_filtered_pos = |app: &App| {
        app.filtered_instance_indices
            .iter()
            .position(|&idx| idx == app.selected_instance_index)
    };

    match code {
        // Navigation - move through filtered items only
        KeyCode::Char('j') | KeyCode::Down => {
            let next_idx = find_filtered_pos(app)
                .and_then(|pos| app.filtered_instance_indices.get(pos + 1).copied())
                .or_else(|| app.filtered_instance_indices.first().copied());
            if let Some(idx) = next_idx {
                update(app, Message::SelectInstance(idx));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev_idx = find_filtered_pos(app)
                .filter(|&pos| pos > 0)
                .and_then(|pos| app.filtered_instance_indices.get(pos - 1).copied())
                .or_else(|| app.filtered_instance_indices.first().copied());
            if let Some(idx) = prev_idx {
                update(app, Message::SelectInstance(idx));
            }
        }
        KeyCode::Char('g') => {
            app.pending_key = Some('g');
        }
        KeyCode::Char('G') | KeyCode::End => {
            if let Some(last) = app.filtered_instance_indices.last().copied() {
                update(app, Message::SelectInstance(last));
            }
        }
        KeyCode::Home => {
            if let Some(first) = app.filtered_instance_indices.first().copied() {
                update(app, Message::SelectInstance(first));
            }
        }

        // Actions
        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
            update(app, Message::LaunchInstance);
        }
        KeyCode::Char('x') => {
            update(app, Message::KillInstance);
        }
        KeyCode::Char('L') => {
            update(app, Message::OpenInstanceLogs);
        }
        KeyCode::Char('s') => {
            update(app, Message::OpenServerScreen);
        }
        KeyCode::Char('S') => {
            update(app, Message::CycleSortMode);
        }
        KeyCode::Char('R') => {
            update(app, Message::ToggleSortDirection);
        }
        KeyCode::Char('a') => {
            update(app, Message::OpenAccountScreen);
        }
        KeyCode::Char('i') => {
            update(app, Message::OpenInstanceDetails);
        }
        KeyCode::Char('o') => {
            update(app, Message::OpenInstanceFolder);
        }
        KeyCode::Tab => {
            update(app, Message::ToggleGroupCollapse);
        }
        KeyCode::Char('/') => {
            update(app, Message::StartSearch);
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                update(app, Message::SearchCancel);
            }
        }
        KeyCode::Char('?') => {
            update(app, Message::OpenHelp);
        }
        KeyCode::Char('q') => {
            update(app, Message::Quit);
        }

        _ => {}
    }
}

fn handle_accounts_key(app: &mut App, code: KeyCode) {
    let find_filtered_pos = |app: &App| {
        app.filtered_account_indices
            .iter()
            .position(|&idx| idx == app.selected_account_index)
    };

    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            let next_idx = find_filtered_pos(app)
                .and_then(|pos| app.filtered_account_indices.get(pos + 1).copied())
                .or_else(|| app.filtered_account_indices.first().copied());
            if let Some(idx) = next_idx {
                update(app, Message::SelectAccount(idx));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev_idx = find_filtered_pos(app)
                .filter(|&pos| pos > 0)
                .and_then(|pos| app.filtered_account_indices.get(pos - 1).copied())
                .or_else(|| app.filtered_account_indices.first().copied());
            if let Some(idx) = prev_idx {
                update(app, Message::SelectAccount(idx));
            }
        }

        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
            update(app, Message::ConfirmAccountSelection);
        }

        KeyCode::Char('h') | KeyCode::Esc | KeyCode::Left => {
            update(app, Message::Back);
        }

        KeyCode::Char('/') => {
            update(app, Message::StartSearch);
        }
        KeyCode::Char('q') => {
            update(app, Message::Quit);
        }

        _ => {}
    }
}

fn handle_servers_key(app: &mut App, code: KeyCode) {
    let total = app.servers.len();

    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if total > 0 && app.selected_server_index + 1 < total {
                update(app, Message::SelectServer(app.selected_server_index + 1));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_server_index > 0 {
                update(app, Message::SelectServer(app.selected_server_index - 1));
            }
        }

        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
            update(app, Message::LaunchWithServer);
        }

        KeyCode::Char('a') => {
            update(app, Message::AddServer);
        }
        KeyCode::Char('e') => {
            update(app, Message::EditServer);
        }
        KeyCode::Char('d') => {
            update(app, Message::DeleteServer);
        }
        KeyCode::Char('J') => {
            update(app, Message::SetJoinOnLaunch);
        }

        KeyCode::Char('h') | KeyCode::Esc | KeyCode::Left => {
            update(app, Message::Back);
        }

        KeyCode::Char('q') => {
            update(app, Message::Quit);
        }

        _ => {}
    }
}

fn handle_details_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('h') | KeyCode::Esc | KeyCode::Left => {
            update(app, Message::Back);
        }
        KeyCode::Char('o') => {
            update(app, Message::OpenInstanceFolder);
        }
        KeyCode::Char('q') => {
            update(app, Message::Quit);
        }
        _ => {}
    }
}

fn handle_help_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            update(app, Message::Back);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            update(app, Message::ScrollHelpDown);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            update(app, Message::ScrollHelpUp);
        }
        _ => {}
    }
}

fn handle_logs_key(app: &mut App, code: KeyCode) {
    let total = app.log_entries.len();

    match code {
        // Navigation in file list
        KeyCode::Char('j') | KeyCode::Down => {
            if total > 0 && app.selected_log_index + 1 < total {
                update(app, Message::SelectLog(app.selected_log_index + 1));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_log_index > 0 {
                update(app, Message::SelectLog(app.selected_log_index - 1));
            }
        }

        // Load selected log content
        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
            update(app, Message::LoadLogContent);
        }

        // Scroll content
        KeyCode::Char('J') | KeyCode::PageDown => {
            update(app, Message::ScrollLogDown(10));
        }
        KeyCode::Char('K') | KeyCode::PageUp => {
            update(app, Message::ScrollLogUp(10));
        }

        // Log search
        KeyCode::Char('/') => {
            update(app, Message::StartLogSearch);
        }
        KeyCode::Char('n') => {
            update(app, Message::LogSearchNext);
        }
        KeyCode::Char('N') => {
            update(app, Message::LogSearchPrev);
        }

        // Log level filtering
        KeyCode::Char('1') => {
            update(app, Message::ToggleLogLevel(LogLevel::Error));
        }
        KeyCode::Char('2') => {
            update(app, Message::ToggleLogLevel(LogLevel::Warn));
        }
        KeyCode::Char('3') => {
            update(app, Message::ToggleLogLevel(LogLevel::Info));
        }
        KeyCode::Char('4') => {
            update(app, Message::ToggleLogLevel(LogLevel::Debug));
        }
        KeyCode::Char('0') => {
            update(app, Message::ShowAllLogLevels);
        }

        // Open in editor
        KeyCode::Char('e') => {
            update(app, Message::OpenLogInEditor);
        }

        // Open folder
        KeyCode::Char('o') => {
            update(app, Message::OpenLogFolder);
        }

        // Back
        KeyCode::Char('h') | KeyCode::Esc | KeyCode::Left => {
            update(app, Message::Back);
        }

        KeyCode::Char('q') => {
            update(app, Message::Quit);
        }

        _ => {}
    }
}

fn toggle_group_collapse(app: &mut App, key: &str) {
    if app.collapsed_groups.contains(key) {
        app.collapsed_groups.remove(key);
    } else {
        app.collapsed_groups.insert(key.to_string());
    }
    let count = app.visible_instance_count();
    app.filtered_instance_indices = (0..count).collect();
    if app.selected_instance_index >= count {
        app.selected_instance_index = count.saturating_sub(1);
    }
}

/// Validate a Minecraft server address
fn validate_server_address(address: &str) -> Result<(), String> {
    if address.is_empty() {
        return Err("Server address cannot be empty".to_string());
    }

    if address.contains(' ') {
        return Err("Server address cannot contain spaces".to_string());
    }

    let parts: Vec<&str> = address.rsplitn(2, ':').collect();
    let host = if parts.len() == 2 {
        if parts[0].parse::<u16>().is_err() {
            return Err("Invalid port number".to_string());
        }
        parts[1]
    } else {
        address
    };

    if host.is_empty() {
        return Err("Server hostname cannot be empty".to_string());
    }

    Ok(())
}

/// Poll running instances by scanning for Java processes matching instance paths.
/// Updates PIDs for tracked instances and removes entries where the game has stopped.
fn poll_running_instances(app: &mut App) {
    let found_pids = scan_java_processes(&mut app.system, &app.instances);

    let mut to_remove = Vec::new();
    for (id, running) in app.running_instances.iter_mut() {
        if let Some(&pid) = found_pids.get(id.as_str()) {
            running.pid = Some(pid);
        } else if running.pid.is_some() {
            // Had a PID but Java process is gone — game exited
            to_remove.push(id.clone());
        } else if running.launched_at.elapsed() > Duration::from_secs(30) {
            // Never found a Java process and it's been too long — give up
            to_remove.push(id.clone());
        }
        // else: recently launched, still waiting for Java to start
    }

    for id in to_remove {
        app.running_instances.remove(&id);
    }
}

/// Scan for Java processes and match them to known instances by path.
fn scan_java_processes(
    system: &mut sysinfo::System,
    instances: &[Instance],
) -> HashMap<String, sysinfo::Pid> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, UpdateKind};

    let refresh_kind = ProcessRefreshKind::nothing().with_cmd(UpdateKind::OnlyIfNotSet);
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);

    let mut result = HashMap::new();

    for (pid, process) in system.processes() {
        let cmd = process.cmd();
        if cmd.is_empty() {
            continue;
        }

        let is_java = cmd.iter().any(|arg| {
            let s = arg.to_string_lossy();
            s.contains("java") || s.ends_with("/java") || s.ends_with("\\java.exe")
        });
        if !is_java {
            continue;
        }

        let full_cmd: String = cmd
            .iter()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        for inst in instances {
            let inst_path = inst.path.to_string_lossy();
            if full_cmd.contains(&*inst_path) {
                result.insert(inst.id.clone(), *pid);
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_server_address_valid() {
        assert!(validate_server_address("mc.hypixel.net").is_ok());
        assert!(validate_server_address("play.example.com:25565").is_ok());
        assert!(validate_server_address("192.168.1.1").is_ok());
        assert!(validate_server_address("192.168.1.1:25565").is_ok());
        assert!(validate_server_address("localhost").is_ok());
        assert!(validate_server_address("localhost:25565").is_ok());
    }

    #[test]
    fn test_validate_server_address_empty() {
        assert!(validate_server_address("").is_err());
    }

    #[test]
    fn test_validate_server_address_spaces() {
        assert!(validate_server_address("example .com").is_err());
        assert!(validate_server_address(" example.com").is_err());
    }

    #[test]
    fn test_validate_server_address_invalid_port() {
        assert!(validate_server_address("example.com:invalid").is_err());
        assert!(validate_server_address("example.com:99999").is_err());
    }

    #[test]
    fn test_validate_server_address_empty_host() {
        assert!(validate_server_address(":25565").is_err());
    }
}
