use crate::app::{LogLevel, Screen};
use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    SwitchToScreen(Screen),

    // Instance actions
    SelectInstance(usize),
    LaunchInstance,
    KillInstance,
    OpenInstanceFolder,
    OpenInstanceDetails,

    // Account actions
    SelectAccount(usize),
    ConfirmAccountSelection,

    // Server actions
    SelectServer(usize),
    AddServer,
    EditServer,
    DeleteServer,
    ConfirmDeleteServer,
    SetJoinOnLaunch,
    LaunchWithServer,

    // Input handling for dialogs
    InputChar(char),
    InputBackspace,
    InputConfirm,
    InputCancel,

    // Screen navigation
    OpenAccountScreen,
    OpenServerScreen,
    OpenInstanceLogs,
    OpenLauncherLogs,
    OpenHelp,
    Back,

    // Log actions
    SelectLog(usize),
    LoadLogContent,
    ScrollLogUp(usize),
    ScrollLogDown(usize),
    OpenLogInEditor,
    OpenLogFolder,

    // Log search
    StartLogSearch,
    LogSearchChar(char),
    LogSearchBackspace,
    LogSearchConfirm,
    LogSearchCancel,
    LogSearchNext,
    LogSearchPrev,

    // Log level filtering
    ToggleLogLevel(LogLevel),
    ShowAllLogLevels,

    // Search
    StartSearch,
    SearchChar(char),
    SearchBackspace,
    SearchConfirm,
    SearchCancel,

    // Sorting
    CycleSortMode,
    ToggleSortDirection,

    // Collapsible groups
    ToggleGroupCollapse,
    NextGroup,
    PrevGroup,

    // Help
    ScrollHelpUp,
    ScrollHelpDown,

    // App control
    Quit,
}
