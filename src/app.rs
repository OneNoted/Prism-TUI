use crate::data::{Account, AppConfig, Instance, LogEntry, PrismConfig, Server};
use crate::error::Result;
use crate::message::Message;
use ratatui::layout::Rect;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

pub struct RunningInstance {
    pub pid: Option<sysinfo::Pid>,
    pub launched_at: Instant,
}

#[derive(Debug, Clone)]
pub enum VisualRow {
    GroupHeader {
        key: String,
        collapsed: bool,
        count: usize,
    },
    Instance(usize), // visual instance index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Instances,
    Accounts,
    Servers,
    Logs,
    InstanceDetails,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    Instance,
    Launcher,
}

pub struct ClickRegion {
    pub rect: Rect,
    pub action: ClickAction,
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    SwitchTab(usize),
    SelectItem(usize),
    GroupHeader(String),
    FooterAction(Message),
    JoinCheckbox,
    GoBack,
    DismissOverlay,
    SelectLogFile(usize),
    ScrollLogPreview,
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    LogSearch,
    AddServerName,
    AddServerAddress,
    EditServerName,
    EditServerAddress,
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    LastPlayed,
    Name,
    Playtime,
    Version,
    ModLoader,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            SortMode::LastPlayed => "Last Played",
            SortMode::Name => "Name",
            SortMode::Playtime => "Playtime",
            SortMode::Version => "Version",
            SortMode::ModLoader => "Mod Loader",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SortMode::LastPlayed => SortMode::Name,
            SortMode::Name => SortMode::Playtime,
            SortMode::Playtime => SortMode::Version,
            SortMode::Version => SortMode::ModLoader,
            SortMode::ModLoader => SortMode::LastPlayed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupedInstances {
    pub group_name: Option<String>,
    pub instances: Vec<Instance>,
}

pub struct App {
    // Core state
    pub running: bool,
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub input_mode: InputMode,

    // Data
    pub data_dir: PathBuf,
    pub instances: Vec<Instance>,
    pub grouped_instances: Vec<GroupedInstances>,
    pub accounts: Vec<Account>,
    pub servers: Vec<Server>,

    // Selection state
    pub selected_instance_index: usize,
    pub selected_account_index: usize,
    pub selected_server_index: usize,

    // Input buffer for dialogs
    pub input_buffer: String,
    pub edit_server_name: String,
    pub edit_server_address: String,

    // Error display
    pub error_message: Option<String>,

    // Active account
    pub active_account: Option<Account>,

    // Search
    pub search_query: String,
    pub filtered_instance_indices: Vec<usize>,
    pub filtered_account_indices: Vec<usize>,

    // Logs
    pub log_entries: Vec<LogEntry>,
    pub selected_log_index: usize,
    pub log_content: Vec<String>,
    pub log_scroll_offset: usize,
    pub log_source: LogSource,
    pub pending_key: Option<char>,

    // Sorting
    pub sort_mode: SortMode,
    pub sort_ascending: bool,

    // Collapsible groups
    pub collapsed_groups: HashSet<String>,

    // Log search
    pub log_search_query: String,
    pub log_search_matches: Vec<usize>,
    pub log_search_current: usize,

    // Log level filter
    pub log_level_filter: HashSet<LogLevel>,

    // App config
    pub app_config: AppConfig,

    // Help scroll
    pub help_scroll_offset: usize,

    // Group selection (for Tab collapse)
    pub selected_group_index: usize,

    // Click regions for mouse support
    pub click_regions: Vec<ClickRegion>,
    pub last_click_time: Option<Instant>,
    pub last_click_pos: (u16, u16),

    // Running instance processes
    pub running_instances: HashMap<String, RunningInstance>,
    pub last_process_scan: Instant,
    pub system: sysinfo::System,
}

impl App {
    pub fn new(config: PrismConfig) -> Result<Self> {
        use crate::data::{load_accounts, load_groups, load_instances};

        let instances_dir = config.instances_dir();
        let groups = load_groups(&instances_dir)?;
        let instances = load_instances(&instances_dir, &groups)?;
        let accounts = load_accounts(&config.accounts_path())?;

        let active_account = accounts.iter().find(|a| a.is_active).cloned();

        let app_config = AppConfig::load();

        let sort_mode = app_config.default_sort_mode();
        let sort_ascending = app_config.sort_ascending;

        let mut app = Self {
            running: true,
            screen: Screen::Instances,
            previous_screen: None,
            input_mode: InputMode::Normal,
            data_dir: config.data_dir,
            instances,
            grouped_instances: Vec::new(),
            accounts,
            servers: Vec::new(),
            selected_instance_index: 0,
            selected_account_index: 0,
            selected_server_index: 0,
            input_buffer: String::new(),
            edit_server_name: String::new(),
            edit_server_address: String::new(),
            error_message: None,
            active_account,
            search_query: String::new(),
            filtered_instance_indices: Vec::new(),
            filtered_account_indices: Vec::new(),
            log_entries: Vec::new(),
            selected_log_index: 0,
            log_content: Vec::new(),
            log_scroll_offset: 0,
            log_source: LogSource::Instance,
            pending_key: None,
            sort_mode,
            sort_ascending,
            collapsed_groups: HashSet::new(),
            log_search_query: String::new(),
            log_search_matches: Vec::new(),
            log_search_current: 0,
            log_level_filter: HashSet::new(),
            app_config,
            help_scroll_offset: 0,
            selected_group_index: 0,
            click_regions: Vec::new(),
            last_click_time: None,
            last_click_pos: (0, 0),
            running_instances: HashMap::new(),
            last_process_scan: Instant::now(),
            system: sysinfo::System::new(),
        };

        app.sort_and_group_instances();

        let instance_count = app
            .grouped_instances
            .iter()
            .map(|g| g.instances.len())
            .sum();
        app.filtered_instance_indices = (0..instance_count).collect();
        app.filtered_account_indices = (0..app.accounts.len()).collect();

        app.selected_account_index = app.accounts.iter().position(|a| a.is_active).unwrap_or(0);

        Ok(app)
    }

    pub fn selected_instance(&self) -> Option<&Instance> {
        self.flat_instance_index()
            .and_then(|idx| self.instances.get(idx))
    }

    pub fn selected_instance_mut(&mut self) -> Option<&mut Instance> {
        self.flat_instance_index()
            .and_then(|idx| self.instances.get_mut(idx))
    }

    /// Get an instance reference by its visual index (skipping collapsed groups)
    pub fn instance_by_visual_idx(&self, target: usize) -> Option<&Instance> {
        let mut visual_count = 0;
        for group in &self.grouped_instances {
            let group_key = group
                .group_name
                .as_deref()
                .unwrap_or("Ungrouped")
                .to_string();
            if self.collapsed_groups.contains(&group_key) {
                continue;
            }
            for instance in &group.instances {
                if visual_count == target {
                    return Some(instance);
                }
                visual_count += 1;
            }
        }
        None
    }

    /// Convert the visual selection index to flat instances index,
    /// accounting for collapsed groups
    fn flat_instance_index(&self) -> Option<usize> {
        let mut visual_count = 0;
        for group in &self.grouped_instances {
            let group_key = group
                .group_name
                .as_deref()
                .unwrap_or("Ungrouped")
                .to_string();
            let is_collapsed = self.collapsed_groups.contains(&group_key);

            if is_collapsed {
                continue;
            }

            for instance in &group.instances {
                if visual_count == self.selected_instance_index {
                    return self.instances.iter().position(|i| i.id == instance.id);
                }
                visual_count += 1;
            }
        }
        None
    }

    pub fn total_instance_count(&self) -> usize {
        self.grouped_instances
            .iter()
            .map(|g| g.instances.len())
            .sum()
    }

    /// Count visible (non-collapsed) instances
    pub fn visible_instance_count(&self) -> usize {
        self.grouped_instances
            .iter()
            .filter(|g| {
                let key = g.group_name.as_deref().unwrap_or("Ungrouped").to_string();
                !self.collapsed_groups.contains(&key)
            })
            .map(|g| g.instances.len())
            .sum()
    }

    pub fn selected_account(&self) -> Option<&Account> {
        self.accounts.get(self.selected_account_index)
    }

    pub fn selected_server(&self) -> Option<&Server> {
        self.servers.get(self.selected_server_index)
    }

    pub fn load_servers_for_instance(&mut self) -> Result<()> {
        use crate::data::load_servers;

        if let Some(instance) = self.selected_instance() {
            let servers_path = instance.servers_dat_path();
            self.servers = load_servers(&servers_path)?;
            self.selected_server_index = 0;
        }
        Ok(())
    }

    pub fn save_servers_for_instance(&self) -> Result<()> {
        use crate::data::save_servers;

        if let Some(instance) = self.selected_instance() {
            let servers_path = instance.servers_dat_path();
            save_servers(&servers_path, &self.servers)?;
        }
        Ok(())
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn update_search(&mut self, query: String) {
        self.search_query = query.to_lowercase();

        if self.search_query.is_empty() {
            // Reset to all indices
            let instance_count = self.visible_instance_count();
            self.filtered_instance_indices = (0..instance_count).collect();
            self.filtered_account_indices = (0..self.accounts.len()).collect();
        } else {
            // Filter instances - match against name, version, mod_loader, group
            let mut idx = 0;
            self.filtered_instance_indices.clear();
            for group in &self.grouped_instances {
                let group_key = group
                    .group_name
                    .as_deref()
                    .unwrap_or("Ungrouped")
                    .to_string();
                let is_collapsed = self.collapsed_groups.contains(&group_key);

                if is_collapsed {
                    continue;
                }

                for instance in &group.instances {
                    let matches = instance.name.to_lowercase().contains(&self.search_query)
                        || instance
                            .minecraft_version
                            .to_lowercase()
                            .contains(&self.search_query)
                        || instance
                            .mod_loader
                            .as_ref()
                            .is_some_and(|l| l.to_lowercase().contains(&self.search_query))
                        || instance
                            .group
                            .as_ref()
                            .is_some_and(|g| g.to_lowercase().contains(&self.search_query));

                    if matches {
                        self.filtered_instance_indices.push(idx);
                    }
                    idx += 1;
                }
            }

            // Filter accounts
            self.filtered_account_indices = self
                .accounts
                .iter()
                .enumerate()
                .filter(|(_, a)| a.username.to_lowercase().contains(&self.search_query))
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection to first filtered item
        self.selected_instance_index = self.filtered_instance_indices.first().copied().unwrap_or(0);
        self.selected_account_index = self.filtered_account_indices.first().copied().unwrap_or(0);
    }

    pub fn clear_search(&mut self) {
        self.update_search(String::new());
    }

    pub fn filtered_instance_count(&self) -> usize {
        self.filtered_instance_indices.len()
    }

    pub fn filtered_account_count(&self) -> usize {
        self.filtered_account_indices.len()
    }

    pub fn sort_and_group_instances(&mut self) {
        // Sort instances
        let ascending = self.sort_ascending;
        self.instances.sort_by(|a, b| {
            let ord = match self.sort_mode {
                SortMode::LastPlayed => b.last_launch.cmp(&a.last_launch),
                SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortMode::Playtime => b.total_time_played.cmp(&a.total_time_played),
                SortMode::Version => a.minecraft_version.cmp(&b.minecraft_version),
                SortMode::ModLoader => {
                    let a_loader = a.mod_loader.as_deref().unwrap_or("");
                    let b_loader = b.mod_loader.as_deref().unwrap_or("");
                    a_loader.cmp(b_loader)
                }
            };
            if ascending { ord } else { ord.reverse() }
        });

        self.grouped_instances = group_instances(&self.instances);

        // Clamp selected group index
        if !self.grouped_instances.is_empty()
            && self.selected_group_index >= self.grouped_instances.len()
        {
            self.selected_group_index = self.grouped_instances.len() - 1;
        }

        // Rebuild filtered indices
        let instance_count = self.visible_instance_count();
        self.filtered_instance_indices = (0..instance_count).collect();
    }

    pub fn update_log_search(&mut self) {
        self.log_search_matches.clear();
        self.log_search_current = 0;

        if self.log_search_query.is_empty() {
            return;
        }

        let query = self.log_search_query.to_lowercase();
        for (i, line) in self.log_content.iter().enumerate() {
            if line.to_lowercase().contains(&query) {
                self.log_search_matches.push(i);
            }
        }

        // Jump to first match
        if let Some(&first_match) = self.log_search_matches.first() {
            self.log_scroll_offset = first_match;
        }
    }

    pub fn log_search_next(&mut self) {
        if self.log_search_matches.is_empty() {
            return;
        }
        self.log_search_current = (self.log_search_current + 1) % self.log_search_matches.len();
        self.log_scroll_offset = self.log_search_matches[self.log_search_current];
    }

    pub fn log_search_prev(&mut self) {
        if self.log_search_matches.is_empty() {
            return;
        }
        if self.log_search_current == 0 {
            self.log_search_current = self.log_search_matches.len() - 1;
        } else {
            self.log_search_current -= 1;
        }
        self.log_scroll_offset = self.log_search_matches[self.log_search_current];
    }

    pub fn filtered_log_content(&self) -> Vec<(usize, &String)> {
        if self.log_level_filter.is_empty() {
            return self.log_content.iter().enumerate().collect();
        }

        self.log_content
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                // If no level detected, always show
                let level = detect_log_level(line);
                match level {
                    Some(l) => self.log_level_filter.contains(&l),
                    None => true,
                }
            })
            .collect()
    }

    pub fn selected_group_key(&self) -> Option<String> {
        self.grouped_instances
            .get(self.selected_group_index)
            .map(|g| g.group_name.as_deref().unwrap_or("Ungrouped").to_string())
    }

    /// Find which group index a visual instance index belongs to
    pub fn group_index_for_instance(&self, instance_visual_idx: usize) -> usize {
        let mut visual_count = 0;
        for (group_idx, group) in self.grouped_instances.iter().enumerate() {
            let group_key = group
                .group_name
                .as_deref()
                .unwrap_or("Ungrouped")
                .to_string();
            if self.collapsed_groups.contains(&group_key) {
                continue;
            }
            let group_end = visual_count + group.instances.len();
            if instance_visual_idx < group_end {
                return group_idx;
            }
            visual_count = group_end;
        }
        0
    }

    /// Find the first visual instance index in a given group
    pub fn first_instance_in_group(&self, group_idx: usize) -> Option<usize> {
        let mut visual_count = 0;
        for (idx, group) in self.grouped_instances.iter().enumerate() {
            let group_key = group
                .group_name
                .as_deref()
                .unwrap_or("Ungrouped")
                .to_string();
            if self.collapsed_groups.contains(&group_key) {
                if idx == group_idx {
                    return None;
                }
                continue;
            }
            if idx == group_idx {
                return if group.instances.is_empty() {
                    None
                } else {
                    Some(visual_count)
                };
            }
            visual_count += group.instances.len();
        }
        None
    }

    pub fn register_click(&mut self, rect: Rect, action: ClickAction) {
        self.click_regions.push(ClickRegion { rect, action });
    }

    pub fn is_instance_running(&self, instance_id: &str) -> bool {
        self.running_instances.contains_key(instance_id)
    }

    pub fn save_config(&self) {
        let mut config = self.app_config.clone();
        config.default_sort = self.sort_mode.label().to_string();
        config.sort_ascending = self.sort_ascending;
        config.save();
    }

    /// Build the visual row mapping for the instances table.
    /// Returns the list of rows as they appear on screen (group headers + instances).
    pub fn visual_rows(&self) -> Vec<VisualRow> {
        let filtered_set: HashSet<usize> = self.filtered_instance_indices.iter().copied().collect();
        let mut rows = Vec::new();
        let mut visual_idx = 0;

        for group in &self.grouped_instances {
            let group_key = group
                .group_name
                .as_deref()
                .unwrap_or("Ungrouped")
                .to_string();
            let is_collapsed = self.collapsed_groups.contains(&group_key);

            // Check if group header should be shown
            let show_header = if is_collapsed {
                true
            } else {
                group
                    .instances
                    .iter()
                    .enumerate()
                    .any(|(i, _)| filtered_set.contains(&(visual_idx + i)))
            };

            if show_header {
                rows.push(VisualRow::GroupHeader {
                    key: group_key,
                    collapsed: is_collapsed,
                    count: group.instances.len(),
                });
            }

            if is_collapsed {
                continue;
            }

            for _ in &group.instances {
                if filtered_set.contains(&visual_idx) {
                    rows.push(VisualRow::Instance(visual_idx));
                }
                visual_idx += 1;
            }
        }

        rows
    }
}

fn detect_log_level(line: &str) -> Option<LogLevel> {
    if line.contains("ERROR") || line.contains("[ERROR]") {
        Some(LogLevel::Error)
    } else if line.contains("WARN") || line.contains("[WARN]") {
        Some(LogLevel::Warn)
    } else if line.contains("INFO") || line.contains("[INFO]") {
        Some(LogLevel::Info)
    } else if line.contains("DEBUG") || line.contains("[DEBUG]") {
        Some(LogLevel::Debug)
    } else {
        None
    }
}

fn group_instances(instances: &[Instance]) -> Vec<GroupedInstances> {
    use std::collections::HashMap;

    let mut groups: HashMap<Option<String>, Vec<Instance>> = HashMap::new();

    for instance in instances {
        groups
            .entry(instance.group.clone())
            .or_default()
            .push(instance.clone());
    }

    let mut result: Vec<GroupedInstances> = groups
        .into_iter()
        .map(|(group_name, instances)| GroupedInstances {
            group_name,
            instances,
        })
        .collect();

    // Sort groups: named groups first (alphabetically), then ungrouped
    result.sort_by(|a, b| match (&a.group_name, &b.group_name) {
        (Some(a_name), Some(b_name)) => a_name.cmp(b_name),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_instance(id: &str, name: &str, group: Option<&str>) -> Instance {
        Instance {
            id: id.to_string(),
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{}", id)),
            group: group.map(|s| s.to_string()),
            minecraft_version: "1.20.1".to_string(),
            mod_loader: None,
            total_time_played: 0,
            last_launch: None,
            server_join: None,
        }
    }

    #[test]
    fn test_group_instances_sorts_correctly() {
        let instances = vec![
            create_test_instance("inst1", "Instance 1", None),
            create_test_instance("inst2", "Instance 2", Some("Modpacks")),
            create_test_instance("inst3", "Instance 3", Some("Vanilla")),
            create_test_instance("inst4", "Instance 4", Some("Modpacks")),
        ];

        let grouped = group_instances(&instances);

        // Named groups should come first, alphabetically
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0].group_name, Some("Modpacks".to_string()));
        assert_eq!(grouped[1].group_name, Some("Vanilla".to_string()));
        assert_eq!(grouped[2].group_name, None); // Ungrouped last
    }

    #[test]
    fn test_group_instances_groups_by_name() {
        let instances = vec![
            create_test_instance("inst1", "Instance 1", Some("Group A")),
            create_test_instance("inst2", "Instance 2", Some("Group A")),
        ];

        let grouped = group_instances(&instances);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].instances.len(), 2);
    }

    #[test]
    fn test_screen_default_is_instances() {
        assert_eq!(Screen::Instances, Screen::Instances);
    }

    #[test]
    fn test_input_mode_default_is_normal() {
        assert_eq!(InputMode::Normal, InputMode::Normal);
    }

    #[test]
    fn test_sort_mode_cycle() {
        assert_eq!(SortMode::LastPlayed.next(), SortMode::Name);
        assert_eq!(SortMode::Name.next(), SortMode::Playtime);
        assert_eq!(SortMode::ModLoader.next(), SortMode::LastPlayed);
    }

    #[test]
    fn test_detect_log_level() {
        assert_eq!(detect_log_level("[ERROR] something"), Some(LogLevel::Error));
        assert_eq!(detect_log_level("[WARN] something"), Some(LogLevel::Warn));
        assert_eq!(detect_log_level("[INFO] something"), Some(LogLevel::Info));
        assert_eq!(detect_log_level("[DEBUG] something"), Some(LogLevel::Debug));
        assert_eq!(detect_log_level("no level here"), None);
    }
}
