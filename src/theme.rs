//! Theme and styling configuration for the LazyLora TUI.
//!
//! This module provides a centralized theme system with Tokyo Night-inspired colors
//! and consistent styling across the application. All color and style constants
//! are encapsulated in the [`Theme`] struct for easy customization and consistency.

use ratatui::style::{Color, Modifier, Style};

// ============================================================================
// Color Constants
// ============================================================================

/// Primary accent color - used for focused elements and highlights.
pub const PRIMARY_COLOR: Color = Color::Cyan;

/// Secondary accent color - used for secondary information.
pub const SECONDARY_COLOR: Color = Color::Blue;

/// Success indicator color - used for positive states and confirmations.
pub const SUCCESS_COLOR: Color = Color::Green;

/// Warning indicator color - used for alerts and caution states.
pub const WARNING_COLOR: Color = Color::Yellow;

/// Error indicator color - used for error states and critical alerts.
pub const ERROR_COLOR: Color = Color::Red;

/// Muted text color - used for secondary/less important text.
pub const MUTED_COLOR: Color = Color::Gray;

/// Accent color - used for special highlights and emphasis.
pub const ACCENT_COLOR: Color = Color::Magenta;

/// Tokyo Night background color - used for subtle overlays and popups.
pub const BG_COLOR: Color = Color::Rgb(26, 27, 38);

// ============================================================================
// Style Constants
// ============================================================================

/// Default border style for unfocused elements.
pub const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);

/// Border style for focused/active elements.
pub const FOCUSED_BORDER_STYLE: Style = Style::new().fg(PRIMARY_COLOR);

/// Title style for unfocused elements.
#[allow(dead_code)] // Design system constant
pub const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

/// Title style for focused/active elements.
pub const FOCUSED_TITLE_STYLE: Style = Style::new().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD);

/// Style for selected items in lists.
pub const SELECTED_STYLE: Style = Style::new().bg(Color::DarkGray);

/// Style for highlighted items with emphasis.
pub const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

// ============================================================================
// Theme Struct
// ============================================================================

/// Application theme containing all colors and styles.
///
/// The theme provides a centralized configuration for the visual appearance
/// of the TUI. It uses Tokyo Night-inspired colors by default but can be
/// customized for different color schemes.
///
/// # Example
///
/// ```rust
/// use lazylora::theme::Theme;
///
/// let theme = Theme::default();
/// let primary = theme.primary;
/// ```
#[allow(dead_code)] // Design system struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Primary accent color for focused elements.
    pub primary: Color,
    /// Secondary accent color for secondary information.
    pub secondary: Color,
    /// Success indicator color.
    pub success: Color,
    /// Warning indicator color.
    pub warning: Color,
    /// Error indicator color.
    pub error: Color,
    /// Muted text color for less important elements.
    pub muted: Color,
    /// Special accent color for highlights.
    pub accent: Color,
    /// Background color for overlays.
    pub background: Color,
    /// Border color for unfocused elements.
    pub border: Color,
    /// Border color for focused elements.
    pub border_focused: Color,
}

impl Theme {
    /// Creates a new theme with custom colors.
    ///
    /// # Arguments
    ///
    /// * `primary` - Primary accent color
    /// * `secondary` - Secondary accent color
    /// * `success` - Success indicator color
    /// * `warning` - Warning indicator color
    /// * `error` - Error indicator color
    /// * `muted` - Muted text color
    /// * `accent` - Special accent color
    /// * `background` - Background color for overlays
    ///
    /// # Returns
    ///
    /// A new `Theme` instance with the specified colors.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn new(
        primary: Color,
        secondary: Color,
        success: Color,
        warning: Color,
        error: Color,
        muted: Color,
        accent: Color,
        background: Color,
    ) -> Self {
        Self {
            primary,
            secondary,
            success,
            warning,
            error,
            muted,
            accent,
            background,
            border: Color::DarkGray,
            border_focused: primary,
        }
    }

    /// Creates the default Tokyo Night-inspired theme.
    ///
    /// # Returns
    ///
    /// A `Theme` instance with Tokyo Night colors.
    #[must_use]
    pub const fn tokyo_night() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Gray,
            accent: Color::Magenta,
            background: Color::Rgb(26, 27, 38),
            border: Color::DarkGray,
            border_focused: Color::Cyan,
        }
    }

    /// Returns the border style based on focus state.
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the element is focused
    ///
    /// # Returns
    ///
    /// A `Style` appropriate for the focus state.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::new().fg(self.border_focused)
        } else {
            Style::new().fg(self.border)
        }
    }

    /// Returns the title style based on focus state.
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the element is focused
    ///
    /// # Returns
    ///
    /// A `Style` appropriate for the focus state.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn title_style(&self, focused: bool) -> Style {
        if focused {
            Style::new().fg(self.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        }
    }

    /// Returns the style for selected items.
    ///
    /// # Returns
    ///
    /// A `Style` for selected list items.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn selected_style(&self) -> Style {
        Style::new().bg(Color::DarkGray)
    }

    /// Returns the style for highlighted items.
    ///
    /// # Returns
    ///
    /// A `Style` for highlighted items with emphasis.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn highlight_style(&self) -> Style {
        Style::new()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns a style with the primary color.
    ///
    /// # Returns
    ///
    /// A `Style` with the primary foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn primary_style(&self) -> Style {
        Style::new().fg(self.primary)
    }

    /// Returns a style with the primary color and bold modifier.
    ///
    /// # Returns
    ///
    /// A bold `Style` with the primary foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn primary_bold_style(&self) -> Style {
        Style::new().fg(self.primary).add_modifier(Modifier::BOLD)
    }

    /// Returns a style with the success color.
    ///
    /// # Returns
    ///
    /// A `Style` with the success foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn success_style(&self) -> Style {
        Style::new().fg(self.success)
    }

    /// Returns a style with the success color and bold modifier.
    ///
    /// # Returns
    ///
    /// A bold `Style` with the success foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn success_bold_style(&self) -> Style {
        Style::new().fg(self.success).add_modifier(Modifier::BOLD)
    }

    /// Returns a style with the warning color.
    ///
    /// # Returns
    ///
    /// A `Style` with the warning foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn warning_style(&self) -> Style {
        Style::new().fg(self.warning)
    }

    /// Returns a style with the warning color and bold modifier.
    ///
    /// # Returns
    ///
    /// A bold `Style` with the warning foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn warning_bold_style(&self) -> Style {
        Style::new().fg(self.warning).add_modifier(Modifier::BOLD)
    }

    /// Returns a style with the error color.
    ///
    /// # Returns
    ///
    /// A `Style` with the error foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn error_style(&self) -> Style {
        Style::new().fg(self.error)
    }

    /// Returns a style with the muted color.
    ///
    /// # Returns
    ///
    /// A `Style` with the muted foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn muted_style(&self) -> Style {
        Style::new().fg(self.muted)
    }

    /// Returns a style with the accent color.
    ///
    /// # Returns
    ///
    /// A `Style` with the accent foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn accent_style(&self) -> Style {
        Style::new().fg(self.accent)
    }

    /// Returns a style with the secondary color.
    ///
    /// # Returns
    ///
    /// A `Style` with the secondary foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn secondary_style(&self) -> Style {
        Style::new().fg(self.secondary)
    }

    /// Returns a style with the secondary color and bold modifier.
    ///
    /// # Returns
    ///
    /// A bold `Style` with the secondary foreground color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn secondary_bold_style(&self) -> Style {
        Style::new().fg(self.secondary).add_modifier(Modifier::BOLD)
    }

    /// Returns a style with the background color.
    ///
    /// # Returns
    ///
    /// A `Style` with the overlay background color.
    #[must_use]
    #[allow(dead_code)] // Part of Theme public API
    pub const fn background_style(&self) -> Style {
        Style::new().bg(self.background)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::tokyo_night()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns the border style based on focus state.
///
/// This is a convenience function that uses the global color constants.
///
/// # Arguments
///
/// * `focused` - Whether the element is focused
///
/// # Returns
///
/// A `Style` appropriate for the focus state.
#[must_use]
#[allow(dead_code)] // Design system utility
pub const fn border_style(focused: bool) -> Style {
    if focused {
        FOCUSED_BORDER_STYLE
    } else {
        BORDER_STYLE
    }
}

/// Returns the title style based on focus state.
///
/// This is a convenience function that uses the global style constants.
///
/// # Arguments
///
/// * `focused` - Whether the element is focused
///
/// # Returns
///
/// A `Style` appropriate for the focus state.
#[must_use]
#[allow(dead_code)] // Design system utility
pub const fn title_style(focused: bool) -> Style {
    if focused {
        FOCUSED_TITLE_STYLE
    } else {
        TITLE_STYLE
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default_equals_tokyo_night() {
        assert_eq!(Theme::default(), Theme::tokyo_night());
    }

    #[test]
    fn test_theme_styles_respond_to_focus() {
        let theme = Theme::default();

        // Border style changes with focus
        let unfocused = theme.border_style(false);
        let focused = theme.border_style(true);
        assert_ne!(unfocused.fg, focused.fg);

        // Title style changes with focus
        let unfocused_title = theme.title_style(false);
        let focused_title = theme.title_style(true);
        assert_ne!(unfocused_title.fg, focused_title.fg);
    }

    #[test]
    fn test_global_helper_functions_match_constants() {
        assert_eq!(border_style(true), FOCUSED_BORDER_STYLE);
        assert_eq!(border_style(false), BORDER_STYLE);
        assert_eq!(title_style(true), FOCUSED_TITLE_STYLE);
        assert_eq!(title_style(false), TITLE_STYLE);
    }
}
