//! Theme and styling constants for the LazyLora TUI.
//!
//! Tokyo Night-inspired colors and consistent styling.

use ratatui::style::{Color, Modifier, Style};

// ============================================================================
// Color Constants
// ============================================================================

/// Primary accent color - focused elements and highlights.
pub const PRIMARY_COLOR: Color = Color::Cyan;

/// Secondary accent color.
pub const SECONDARY_COLOR: Color = Color::Blue;

/// Success indicator color.
pub const SUCCESS_COLOR: Color = Color::Green;

/// Warning indicator color.
pub const WARNING_COLOR: Color = Color::Yellow;

/// Error indicator color.
pub const ERROR_COLOR: Color = Color::Red;

/// Muted text color.
pub const MUTED_COLOR: Color = Color::Gray;

/// Accent color for special highlights.
pub const ACCENT_COLOR: Color = Color::Magenta;

/// Tokyo Night background color.
pub const BG_COLOR: Color = Color::Rgb(26, 27, 38);

// ============================================================================
// Style Constants
// ============================================================================

/// Default border style for unfocused elements.
pub const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);

/// Border style for focused/active elements.
pub const FOCUSED_BORDER_STYLE: Style = Style::new().fg(PRIMARY_COLOR);

/// Title style for focused/active elements.
pub const FOCUSED_TITLE_STYLE: Style = Style::new().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD);

/// Style for selected items in lists.
pub const SELECTED_STYLE: Style = Style::new().bg(Color::DarkGray);

/// Style for highlighted items with emphasis.
pub const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);
