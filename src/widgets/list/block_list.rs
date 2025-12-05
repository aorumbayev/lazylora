//! Block list widget.
//!
//! Displays a list of Algorand blocks with selection and scrolling support.

#![allow(dead_code)] // Transitional phase - items will be used after integration

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::domain::AlgoBlock;

use super::state::{BlockListState, render_list_scrollbar};

// ============================================================================
// BlockListWidget
// ============================================================================

/// A widget that displays a list of blocks with selection and scrolling.
///
/// This widget implements `StatefulWidget` and requires a `BlockListState`
/// to track selection and scroll position.
///
/// # Example
///
/// ```text
/// ┌─ Latest Blocks ─────────────────────┐
/// │ ▶ 12345678          15 txns         │
/// │   Mon, 01 Jan 2024 12:00:00         │
/// │                                     │
/// │ ⬚ 12345677          8 txns          │
/// │   Mon, 01 Jan 2024 11:59:55         │
/// │                                     │
/// └─────────────────────────────────────┘
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::list::{BlockListWidget, BlockListState};
///
/// let blocks = vec![/* ... */];
/// let widget = BlockListWidget::new(&blocks).focused(true);
/// let mut state = BlockListState::with_selection(0);
///
/// // Render with frame.render_stateful_widget(widget, area, &mut state);
/// ```
#[derive(Debug)]
pub struct BlockListWidget<'a> {
    /// Slice of blocks to display.
    blocks: &'a [AlgoBlock],
    /// Whether this widget is currently focused.
    focused: bool,
    /// Height of each block item in rows.
    item_height: u16,
}

impl<'a> BlockListWidget<'a> {
    /// Height of each block item in the list (in rows).
    pub const DEFAULT_ITEM_HEIGHT: u16 = 3;

    /// Creates a new `BlockListWidget` with the given blocks.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A slice of `AlgoBlock` items to display
    ///
    /// # Returns
    ///
    /// A new `BlockListWidget` with default settings
    #[must_use]
    pub const fn new(blocks: &'a [AlgoBlock]) -> Self {
        Self {
            blocks,
            focused: false,
            item_height: Self::DEFAULT_ITEM_HEIGHT,
        }
    }

    /// Sets whether this widget is focused.
    ///
    /// When focused, the scrollbar will be rendered if content exceeds the viewport.
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the widget should be considered focused
    ///
    /// # Returns
    ///
    /// Self with the focus state updated
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the item height.
    ///
    /// # Arguments
    ///
    /// * `height` - The height of each item in rows
    ///
    /// # Returns
    ///
    /// Self with the item height updated
    #[must_use]
    pub const fn item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Returns the number of blocks.
    ///
    /// # Returns
    ///
    /// The number of blocks in the list
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns true if there are no blocks.
    ///
    /// # Returns
    ///
    /// `true` if the blocks slice is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl StatefulWidget for BlockListWidget<'_> {
    type State = BlockListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Handle empty state
        if self.blocks.is_empty() {
            let empty_msg = "No blocks available";
            let x = area.x + (area.width.saturating_sub(empty_msg.len() as u16)) / 2;
            let y = area.y + area.height / 2;

            if y < area.y + area.height && x < area.x + area.width {
                let style = Style::default().fg(Color::Gray);
                buf.set_string(x, y, empty_msg, style);
            }
            return;
        }

        // Build list items
        let block_items: Vec<ListItem> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let is_selected = state.selected_index == Some(i);
                let selection_indicator = if is_selected { "▶" } else { "⬚" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(format!("{selection_indicator} ")),
                        Span::styled(
                            block.id.to_string(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("               "),
                        Span::styled(
                            format!("{} txns", block.txn_count),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&block.timestamp, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(""),
                ])
                .style(if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            })
            .collect();

        let items_per_page = area.height as usize / self.item_height as usize;
        let block_scroll_usize = state.scroll_position as usize / self.item_height as usize;
        let start_index = block_scroll_usize.min(self.blocks.len().saturating_sub(1));
        let end_index = (start_index + items_per_page).min(self.blocks.len());
        let visible_items = block_items[start_index..end_index].to_vec();

        // Create internal ListState for highlighting
        let mut list_state = ListState::default();
        if let Some(selected) = state.selected_index
            && selected >= start_index
            && selected < end_index
        {
            list_state.select(Some(selected - start_index));
        }

        let block_list = List::new(visible_items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        StatefulWidget::render(block_list, area, buf, &mut list_state);

        // Render scrollbar if focused and content exceeds viewport
        if self.focused && self.blocks.len() > items_per_page {
            render_list_scrollbar(
                area,
                buf,
                self.blocks.len(),
                self.item_height as usize,
                items_per_page,
                state.scroll_position as usize,
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_blocks() -> Vec<AlgoBlock> {
        vec![
            AlgoBlock {
                id: 12345678,
                txn_count: 15,
                timestamp: "Mon, 01 Jan 2024 12:00:00".to_string(),
            },
            AlgoBlock {
                id: 12345677,
                txn_count: 8,
                timestamp: "Mon, 01 Jan 2024 11:59:55".to_string(),
            },
            AlgoBlock {
                id: 12345676,
                txn_count: 22,
                timestamp: "Mon, 01 Jan 2024 11:59:50".to_string(),
            },
        ]
    }

    #[test]
    fn test_block_list_widget_new() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);

        assert_eq!(widget.len(), 3);
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_empty() {
        let blocks: Vec<AlgoBlock> = vec![];
        let widget = BlockListWidget::new(&blocks);

        assert_eq!(widget.len(), 0);
        assert!(widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_focused() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks).focused(true);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_item_height() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks).item_height(5);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_render_empty() {
        let blocks: Vec<AlgoBlock> = vec![];
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::new();

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render "No blocks available" message
        let content = buf_to_string(&buf);
        assert!(content.contains("No blocks available"));
    }

    #[test]
    fn test_block_list_widget_render_with_blocks() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::new();

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render block IDs
        let content = buf_to_string(&buf);
        assert!(content.contains("12345678"));
    }

    #[test]
    fn test_block_list_widget_render_with_selection() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::with_selection(0);

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render selection indicator for first item
        let content = buf_to_string(&buf);
        assert!(content.contains("▶")); // Selected indicator
    }

    // Helper function to convert buffer to string for testing
    fn buf_to_string(buf: &Buffer) -> String {
        let area = buf.area;
        let mut result = String::new();

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell((x, y)) {
                    result.push_str(cell.symbol());
                }
            }
            result.push('\n');
        }

        result
    }
}
