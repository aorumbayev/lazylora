//! Layout calculations for the LazyLora TUI.

use ratatui::layout::Rect;

// ============================================================================
// Constants
// ============================================================================

/// Height of the header area in terminal rows.
pub const HEADER_HEIGHT: u16 = 3;

/// Height of the title/explore section.
pub const TITLE_HEIGHT: u16 = 3;

// ============================================================================
// Layout Functions
// ============================================================================

/// Calculate a centered popup area within a parent area.
#[must_use]
pub fn centered_popup_area(parent: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(parent.width.saturating_sub(4));
    let popup_height = height.min(parent.height.saturating_sub(4));

    let popup_x = parent.x + (parent.width.saturating_sub(popup_width)) / 2;
    let popup_y = parent.y + (parent.height.saturating_sub(popup_height)) / 2;

    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_popup_area() {
        let cases = [
            // (parent, width, height, expected)
            (Rect::new(0, 0, 100, 50), 40, 20, Rect::new(30, 15, 40, 20)),
            (Rect::new(0, 0, 30, 20), 100, 50, Rect::new(2, 2, 26, 16)),
        ];

        for (parent, width, height, expected) in cases {
            let result = centered_popup_area(parent, width, height);
            assert_eq!(result, expected, "parent={parent:?}, w={width}, h={height}");
        }
    }
}
