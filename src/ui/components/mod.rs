//! Reusable UI components for the LazyLora TUI.
//!
//! This module contains standalone, reusable components that can be composed
//! to build the application UI. Components are designed to be stateless and
//! focus on rendering specific UI elements.
//!
//! # Components
//!
//! - [`toast`] - Toast notification overlay for non-blocking messages
//!
//! # Design Philosophy
//!
//! Components in this module follow these principles:
//! - **Stateless**: Components don't manage application state
//! - **Composable**: Can be combined to create complex UIs
//! - **Testable**: Each component has comprehensive unit tests
//! - **Documented**: Public APIs have clear documentation and examples
//!
//! # Example
//!
//! ```rust,no_run
//! use ratatui::Frame;
//! use lazylora::ui::components::toast::render_toast;
//!
//! fn render_with_notification(frame: &mut Frame) {
//!     let area = frame.area();
//!     // Render main UI...
//!     
//!     // Show notification overlay
//!     render_toast(frame, area, "✓ Operation complete");
//! }
//! ```

pub mod toast;

// Re-export commonly used component functions for convenience
pub use toast::render_toast;

// ============================================================================
// Shared Component Utilities
// ============================================================================

/// Common component configuration and utility functions.
///
/// This module can be expanded to include shared helpers like:
/// - Color theme helpers
/// - Common layout calculations
/// - Shared rendering patterns
#[allow(dead_code)]
pub mod common {
    use ratatui::layout::Rect;

    /// Calculates a centered rectangle within a given area.
    ///
    /// # Arguments
    ///
    /// * `area` - The containing area
    /// * `width` - Desired width (will be clamped to area width)
    /// * `height` - Desired height (will be clamped to area height)
    ///
    /// # Returns
    ///
    /// A `Rect` centered within the given area.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::layout::Rect;
    /// use lazylora::ui::components::common::centered_rect;
    ///
    /// let area = Rect::new(0, 0, 100, 50);
    /// let centered = centered_rect(area, 40, 20);
    /// assert_eq!(centered.width, 40);
    /// assert_eq!(centered.height, 20);
    /// ```
    #[must_use]
    pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
        let width = width.min(area.width);
        let height = height.min(area.height);

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    /// Calculates a rectangle positioned in a specific corner of an area.
    ///
    /// # Arguments
    ///
    /// * `area` - The containing area
    /// * `width` - Width of the corner rectangle
    /// * `height` - Height of the corner rectangle
    /// * `corner` - Which corner to position in
    /// * `padding_x` - Horizontal padding from edge
    /// * `padding_y` - Vertical padding from edge
    ///
    /// # Returns
    ///
    /// A `Rect` positioned in the specified corner with padding.
    #[must_use]
    pub fn corner_rect(
        area: Rect,
        width: u16,
        height: u16,
        corner: Corner,
        padding_x: u16,
        padding_y: u16,
    ) -> Rect {
        let (x, y) = match corner {
            Corner::TopLeft => (area.x + padding_x, area.y + padding_y),
            Corner::TopRight => (
                area.x + area.width.saturating_sub(width + padding_x),
                area.y + padding_y,
            ),
            Corner::BottomLeft => (
                area.x + padding_x,
                area.y + area.height.saturating_sub(height + padding_y),
            ),
            Corner::BottomRight => (
                area.x + area.width.saturating_sub(width + padding_x),
                area.y + area.height.saturating_sub(height + padding_y),
            ),
        };

        Rect::new(x, y, width, height)
    }

    /// Represents a corner position within a rectangle.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Corner {
        /// Top-left corner.
        TopLeft,
        /// Top-right corner.
        TopRight,
        /// Bottom-left corner.
        BottomLeft,
        /// Bottom-right corner.
        BottomRight,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::common::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(area, 40, 20);

        assert_eq!(centered.width, 40);
        assert_eq!(centered.height, 20);
        assert_eq!(centered.x, 30); // (100 - 40) / 2
        assert_eq!(centered.y, 15); // (50 - 20) / 2
    }

    #[test]
    fn test_centered_rect_oversized() {
        let area = Rect::new(0, 0, 50, 30);
        let centered = centered_rect(area, 100, 60);

        // Should be clamped to area size
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 30);
        assert_eq!(centered.x, 0);
        assert_eq!(centered.y, 0);
    }

    #[test]
    fn test_centered_rect_exact_fit() {
        let area = Rect::new(10, 5, 50, 30);
        let centered = centered_rect(area, 50, 30);

        assert_eq!(centered, area);
    }

    #[test]
    fn test_corner_rect_bottom_right() {
        let area = Rect::new(0, 0, 100, 50);
        let corner = corner_rect(area, 20, 10, Corner::BottomRight, 2, 2);

        assert_eq!(corner.width, 20);
        assert_eq!(corner.height, 10);
        assert_eq!(corner.x, 78); // 100 - 20 - 2
        assert_eq!(corner.y, 38); // 50 - 10 - 2
    }

    #[test]
    fn test_corner_rect_top_left() {
        let area = Rect::new(0, 0, 100, 50);
        let corner = corner_rect(area, 20, 10, Corner::TopLeft, 5, 3);

        assert_eq!(corner.x, 5);
        assert_eq!(corner.y, 3);
    }

    #[test]
    fn test_corner_rect_top_right() {
        let area = Rect::new(0, 0, 100, 50);
        let corner = corner_rect(area, 20, 10, Corner::TopRight, 2, 3);

        assert_eq!(corner.x, 78); // 100 - 20 - 2
        assert_eq!(corner.y, 3);
    }

    #[test]
    fn test_corner_rect_bottom_left() {
        let area = Rect::new(0, 0, 100, 50);
        let corner = corner_rect(area, 20, 10, Corner::BottomLeft, 5, 2);

        assert_eq!(corner.x, 5);
        assert_eq!(corner.y, 38); // 50 - 10 - 2
    }

    #[test]
    fn test_corner_enum_equality() {
        assert_eq!(Corner::TopLeft, Corner::TopLeft);
        assert_ne!(Corner::TopLeft, Corner::TopRight);
    }
}
