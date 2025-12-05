//! Block list widget.
//!
//! Displays a list of Algorand blocks with selection and scrolling support.

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
#[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns true if there are no blocks.
    ///
    /// # Returns
    ///
    /// `true` if the blocks slice is empty
    #[must_use]
    #[allow(dead_code)]
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

    /// Per commandments: test observable behavior, not construction.
    /// Consolidated test for widget properties.
    #[test]
    fn test_block_list_widget_properties() {
        let blocks = create_sample_blocks();
        let empty: Vec<AlgoBlock> = vec![];

        // Test with data
        let widget = BlockListWidget::new(&blocks).focused(true).item_height(5);
        assert_eq!(widget.len(), 3);
        assert!(!widget.is_empty());

        // Test empty
        let empty_widget = BlockListWidget::new(&empty);
        assert!(empty_widget.is_empty());
        assert_eq!(empty_widget.len(), 0);
    }

    /// Consolidated rendering test - tests observable output, not internals.
    /// Per commandments: "One happy path snapshot > Five edge case unit tests"
    #[test]
    fn test_block_list_rendering_states() {
        let blocks = create_sample_blocks();
        let area = Rect::new(0, 0, 60, 20);

        // Empty state
        let mut buf = Buffer::empty(area);
        let mut state = BlockListState::new();
        BlockListWidget::new(&[]).render(area, &mut buf, &mut state);
        let content = buf_to_string(&buf);
        assert!(
            content.contains("No blocks available"),
            "empty state message"
        );

        // With data and selection
        let mut buf = Buffer::empty(area);
        let mut state = BlockListState::with_selection(0);
        BlockListWidget::new(&blocks).render(area, &mut buf, &mut state);
        let content = buf_to_string(&buf);
        assert!(content.contains("12345678"), "renders block ID");
        assert!(content.contains("▶"), "renders selection indicator");
    }

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
