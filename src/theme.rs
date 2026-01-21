use ratatui::style::Color;

/// Catppuccin Mocha color palette
pub mod colors {
    use super::Color;

    // Base colors
    pub const BASE: Color = Color::Rgb(30, 30, 46);
    pub const MANTLE: Color = Color::Rgb(24, 24, 37);
    pub const CRUST: Color = Color::Rgb(17, 17, 27);

    // Surface colors
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
    pub const SURFACE2: Color = Color::Rgb(88, 91, 112);

    // Overlay colors
    pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);
    pub const OVERLAY1: Color = Color::Rgb(127, 132, 156);
    pub const OVERLAY2: Color = Color::Rgb(147, 153, 178);

    // Text colors
    pub const TEXT: Color = Color::Rgb(205, 214, 244);
    pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
    pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);

    // Accent colors
    pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);
    pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);
    pub const PINK: Color = Color::Rgb(245, 194, 231);
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);
    pub const RED: Color = Color::Rgb(243, 139, 168);
    pub const MAROON: Color = Color::Rgb(235, 160, 172);
    pub const PEACH: Color = Color::Rgb(250, 179, 135);
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);
    pub const GREEN: Color = Color::Rgb(166, 227, 161);
    pub const TEAL: Color = Color::Rgb(148, 226, 213);
    pub const SKY: Color = Color::Rgb(137, 220, 235);
    pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
    pub const BLUE: Color = Color::Rgb(137, 180, 250);
    pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
}

/// Semantic color aliases for UI elements
pub mod ui {
    use super::Color;
    use super::colors;

    /// Primary accent color for titles and selected items
    pub const PRIMARY: Color = colors::MAUVE;

    /// Secondary accent for active/enabled states
    pub const ACTIVE: Color = colors::GREEN;

    /// Search and highlight color
    pub const HIGHLIGHT: Color = colors::YELLOW;

    /// Error states and messages
    pub const ERROR: Color = colors::RED;

    /// Warning log level
    pub const WARNING: Color = colors::YELLOW;

    /// Info log level
    pub const INFO: Color = colors::BLUE;

    /// Mode indicator (vim/arrows)
    pub const MODE: Color = colors::MAUVE;

    /// Muted/secondary text
    pub const MUTED: Color = colors::OVERLAY0;

    /// Normal text color
    pub const TEXT: Color = colors::TEXT;

    /// Debug log level
    pub const DEBUG: Color = colors::OVERLAY0;

    /// Dialog borders
    pub const DIALOG_BORDER: Color = colors::YELLOW;

    /// Help dialog border
    pub const HELP_BORDER: Color = colors::MAUVE;
}
