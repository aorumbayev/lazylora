//! Layout calculations for the LazyLora TUI
//!
//! This module provides layout structs and helper functions for
//! calculating UI element positions and sizes.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

// ============================================================================
// Constants
// ============================================================================

/// Height of the header area in terminal rows
pub const HEADER_HEIGHT: u16 = 3;

/// Height of the title/explore section
pub const TITLE_HEIGHT: u16 = 3;

/// Height of the footer area in terminal rows
#[allow(dead_code)] // Design system constant
pub const FOOTER_HEIGHT: u16 = 1;

/// Height of each block item in the list
#[allow(dead_code)] // Design system constant
pub const BLOCK_HEIGHT: u16 = 3;

/// Height of each transaction item in the list
#[allow(dead_code)] // Design system constant
pub const TXN_HEIGHT: u16 = 4;

// ============================================================================
// Layout Structs
// ============================================================================

/// Main application layout areas
#[allow(dead_code)] // Design system struct
#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    /// Header area (logo, network status)
    pub header: Rect,
    /// Main content area (panels or details)
    pub main: Rect,
    /// Footer area (keybinding hints)
    pub footer: Rect,
}

/// Two-panel layout for main content area
#[allow(dead_code)] // Design system struct
#[derive(Debug, Clone, Copy)]
pub struct MainLayout {
    /// Title/explore section with live toggle
    pub title: Rect,
    /// Content area below title
    pub content: Rect,
}

/// Left/right panel layout for content
#[allow(dead_code)] // Design system struct
#[derive(Debug, Clone, Copy)]
pub struct PanelLayout {
    /// Left panel (blocks)
    pub left: Rect,
    /// Right panel (transactions)
    pub right: Rect,
}

/// Popup layout with optional tab bar
#[allow(dead_code)] // Design system struct
#[derive(Debug, Clone, Copy)]
pub struct PopupLayout {
    /// Tab bar area (if present)
    pub tabs: Option<Rect>,
    /// Separator line
    pub separator: Option<Rect>,
    /// Main content area
    pub content: Rect,
    /// Action buttons area
    pub actions: Option<Rect>,
    /// Help text area
    pub help: Rect,
}

// ============================================================================
// Layout Functions
// ============================================================================

/// Calculate the main application layout from the terminal area
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn calculate_app_layout(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(3),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    AppLayout {
        header: chunks[0],
        main: chunks[1],
        footer: chunks[2],
    }
}

/// Calculate the main content layout (title + content areas)
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn calculate_main_layout(area: Rect) -> MainLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(TITLE_HEIGHT), Constraint::Min(10)])
        .split(area);

    MainLayout {
        title: chunks[0],
        content: chunks[1],
    }
}

/// Calculate the two-panel layout for blocks and transactions
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn calculate_panel_layout(area: Rect) -> PanelLayout {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    PanelLayout {
        left: chunks[0],
        right: chunks[1],
    }
}

/// Calculate a popup layout with tabs, content, actions, and help
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn calculate_popup_layout(area: Rect, has_tabs: bool, has_actions: bool) -> PopupLayout {
    let mut constraints = Vec::new();

    if has_tabs {
        constraints.push(Constraint::Length(1)); // Tab bar
        constraints.push(Constraint::Length(1)); // Separator
    }

    constraints.push(Constraint::Min(6)); // Main content

    if has_actions {
        constraints.push(Constraint::Length(3)); // Action buttons
    }

    constraints.push(Constraint::Length(1)); // Help text

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    let tabs = if has_tabs {
        let t = Some(chunks[idx]);
        idx += 1;
        t
    } else {
        None
    };

    let separator = if has_tabs {
        let s = Some(chunks[idx]);
        idx += 1;
        s
    } else {
        None
    };

    let content = chunks[idx];
    idx += 1;

    let actions = if has_actions {
        let a = Some(chunks[idx]);
        idx += 1;
        a
    } else {
        None
    };

    let help = chunks[idx];

    PopupLayout {
        tabs,
        separator,
        content,
        actions,
        help,
    }
}

/// Calculate a centered popup area within a parent area
///
/// # Arguments
/// * `parent` - The parent area to center within
/// * `width` - Desired popup width in columns
/// * `height` - Desired popup height in rows
#[must_use]
pub fn centered_popup_area(parent: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(parent.width.saturating_sub(4));
    let popup_height = height.min(parent.height.saturating_sub(4));

    let popup_x = parent.x + (parent.width.saturating_sub(popup_width)) / 2;
    let popup_y = parent.y + (parent.height.saturating_sub(popup_height)) / 2;

    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

/// Calculate a centered rect as a percentage of the parent
///
/// # Arguments
/// * `percent_x` - Width as percentage of parent (0-100)
/// * `percent_y` - Height as percentage of parent (0-100)
/// * `area` - Parent area
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn centered_rect_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let height = area.height * percent_y / 100;
    centered_popup_area(area, width, height)
}

/// Calculate the display width of a string in characters
#[must_use]
#[allow(dead_code)] // Design system utility
pub fn string_width(s: &str) -> u16 {
    s.chars().count() as u16
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_app_layout() {
        let area = Rect::new(0, 0, 100, 50);
        let layout = calculate_app_layout(area);

        assert_eq!(layout.header.height, HEADER_HEIGHT);
        assert_eq!(layout.footer.height, FOOTER_HEIGHT);
        assert_eq!(layout.main.height, 50 - HEADER_HEIGHT - FOOTER_HEIGHT);
    }

    #[test]
    fn test_calculate_main_layout() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = calculate_main_layout(area);

        assert_eq!(layout.title.height, TITLE_HEIGHT);
        assert_eq!(layout.content.height, 40 - TITLE_HEIGHT);
    }

    #[test]
    fn test_calculate_panel_layout() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = calculate_panel_layout(area);

        assert_eq!(layout.left.width, 50);
        assert_eq!(layout.right.width, 50);
        assert_eq!(layout.left.height, layout.right.height);
    }

    #[test]
    fn test_centered_popup_area() {
        let parent = Rect::new(0, 0, 100, 50);
        let popup = centered_popup_area(parent, 40, 20);

        assert_eq!(popup.width, 40);
        assert_eq!(popup.height, 20);
        assert_eq!(popup.x, 30); // (100 - 40) / 2
        assert_eq!(popup.y, 15); // (50 - 20) / 2
    }

    #[test]
    fn test_centered_popup_area_clamped() {
        let parent = Rect::new(0, 0, 30, 20);
        let popup = centered_popup_area(parent, 100, 50);

        // Should be clamped to fit within parent with margin
        assert!(popup.width <= parent.width - 4);
        assert!(popup.height <= parent.height - 4);
    }

    #[test]
    fn test_centered_rect_percent() {
        let area = Rect::new(0, 0, 100, 50);
        let rect = centered_rect_percent(50, 50, area);

        assert_eq!(rect.width, 50);
        assert_eq!(rect.height, 25);
    }

    #[test]
    fn test_calculate_popup_layout_with_tabs() {
        let area = Rect::new(0, 0, 80, 30);
        let layout = calculate_popup_layout(area, true, true);

        assert!(layout.tabs.is_some());
        assert!(layout.separator.is_some());
        assert!(layout.actions.is_some());
        assert_eq!(layout.help.height, 1);
    }

    #[test]
    fn test_calculate_popup_layout_minimal() {
        let area = Rect::new(0, 0, 80, 30);
        let layout = calculate_popup_layout(area, false, false);

        assert!(layout.tabs.is_none());
        assert!(layout.separator.is_none());
        assert!(layout.actions.is_none());
        assert_eq!(layout.help.height, 1);
    }

    #[test]
    fn test_string_width() {
        assert_eq!(string_width("hello"), 5);
        assert_eq!(string_width(""), 0);
        assert_eq!(string_width("hello world"), 11);
    }
}
