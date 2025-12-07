//! Visual/graph view rendering for transaction details.
//!
//! Handles the graphical representation of transactions using `TxnGraph` and `TxnVisualCard`.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::domain::Transaction;
use crate::state::App;
use crate::theme::BG_COLOR;
use crate::widgets::{TxnGraph, TxnGraphWidget, TxnVisualCard};

// ============================================================================
// Visual Mode Rendering
// ============================================================================

/// Renders the transaction in visual/graph mode.
///
/// Returns `true` if scrolling is needed (content exceeds viewport).
pub fn render_visual_mode(
    app: &App,
    txn: &Transaction,
    graph: &TxnGraph,
    graph_widget: &TxnGraphWidget,
    frame: &mut Frame,
    area: Rect,
) -> bool {
    let graph_lines = graph_widget.to_lines();

    // If graph has meaningful content, show it
    if !graph.columns.is_empty() {
        // Calculate padded area (minimal padding for compactness)
        let padded_area = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );

        // Calculate graph dimensions (use required_width for accurate measurement)
        let graph_height = graph_widget.required_height();
        let graph_width = graph_widget.required_width();

        // Determine if we need scrolling
        let needs_v_scroll = graph_height > padded_area.height as usize;
        let needs_h_scroll = graph_width > padded_area.width as usize;

        // Calculate max scroll values
        let max_scroll_y = graph_height.saturating_sub(padded_area.height as usize);
        let max_scroll_x = graph_width.saturating_sub(padded_area.width as usize);

        // Get scroll offsets from app state, clamped to valid range
        let scroll_x = (app.nav.graph_scroll_x as usize).min(max_scroll_x);
        let scroll_y = (app.nav.graph_scroll_y as usize).min(max_scroll_y);

        // Calculate centering offsets (when graph fits in view)
        let center_x = if !needs_h_scroll {
            (padded_area.width as usize).saturating_sub(graph_width) / 2
        } else {
            0
        };
        let center_y = if !needs_v_scroll {
            (padded_area.height as usize).saturating_sub(graph_height) / 2
        } else {
            0
        };

        // Build visible lines with centering or scrolling
        let visible_lines: Vec<Line> = if needs_v_scroll || needs_h_scroll {
            // Scrolling mode - apply scroll offsets
            graph_lines
                .into_iter()
                .skip(scroll_y)
                .take(padded_area.height as usize)
                .map(|line| {
                    if scroll_x > 0 {
                        let mut remaining_skip = scroll_x;
                        let mut new_spans = Vec::new();

                        for span in line.spans {
                            let content = span.content.to_string();
                            let char_count = content.chars().count();

                            if remaining_skip >= char_count {
                                remaining_skip -= char_count;
                                continue;
                            }

                            if remaining_skip > 0 {
                                let new_content: String =
                                    content.chars().skip(remaining_skip).collect();
                                new_spans.push(Span::styled(new_content, span.style));
                                remaining_skip = 0;
                            } else {
                                new_spans.push(Span::styled(content, span.style));
                            }
                        }

                        Line::from(new_spans)
                    } else {
                        line
                    }
                })
                .collect()
        } else {
            // Centering mode - add padding for both horizontal and vertical centering
            let mut centered_lines: Vec<Line> = Vec::new();

            // Add top padding for vertical centering
            centered_lines.extend(std::iter::repeat_n(Line::from(""), center_y));

            // Add graph lines with horizontal centering
            for line in graph_lines {
                if center_x > 0 {
                    let mut new_spans = vec![Span::raw(" ".repeat(center_x))];
                    new_spans.extend(line.spans);
                    centered_lines.push(Line::from(new_spans));
                } else {
                    centered_lines.push(line);
                }
            }

            centered_lines
        };

        let visual_content = Paragraph::new(visible_lines).alignment(Alignment::Left);
        frame.render_widget(visual_content, padded_area);

        // Show scroll indicator ONLY if scrolling is needed
        if needs_v_scroll || needs_h_scroll {
            render_scroll_indicator(
                frame,
                padded_area,
                needs_v_scroll,
                needs_h_scroll,
                scroll_y,
                scroll_x,
                max_scroll_y,
                max_scroll_x,
            );
        }

        needs_v_scroll || needs_h_scroll
    } else {
        // Fallback to TxnVisualCard for edge cases
        let visual_card = TxnVisualCard::new(txn);
        let lines = visual_card.to_lines();

        let visual_content = Paragraph::new(lines).alignment(Alignment::Left);

        let padded_area = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );
        frame.render_widget(visual_content, padded_area);

        false // Fallback mode doesn't support scrolling
    }
}

// ============================================================================
// Scroll Indicator
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn render_scroll_indicator(
    frame: &mut Frame,
    area: Rect,
    needs_v_scroll: bool,
    needs_h_scroll: bool,
    scroll_y: usize,
    scroll_x: usize,
    max_scroll_y: usize,
    max_scroll_x: usize,
) {
    // Build a compact scroll indicator
    let scroll_hint = if needs_v_scroll && needs_h_scroll {
        // Show position with directional arrows
        let v_indicator = if scroll_y > 0 && scroll_y < max_scroll_y {
            "↕"
        } else if scroll_y > 0 {
            "↑"
        } else {
            "↓"
        };
        let h_indicator = if scroll_x > 0 && scroll_x < max_scroll_x {
            "↔"
        } else if scroll_x > 0 {
            "←"
        } else {
            "→"
        };
        format!(" {} {} ", v_indicator, h_indicator)
    } else if needs_v_scroll {
        let v_indicator = if scroll_y > 0 && scroll_y < max_scroll_y {
            "↕"
        } else if scroll_y > 0 {
            "↑"
        } else {
            "↓"
        };
        format!(" {} ", v_indicator)
    } else {
        let h_indicator = if scroll_x > 0 && scroll_x < max_scroll_x {
            "↔"
        } else if scroll_x > 0 {
            "←"
        } else {
            "→"
        };
        format!(" {} ", h_indicator)
    };

    let hint_width = scroll_hint.chars().count() as u16;
    let hint_area = Rect::new(
        area.x + area.width.saturating_sub(hint_width + 1),
        area.y + area.height.saturating_sub(1),
        hint_width,
        1,
    );

    let hint_widget =
        Paragraph::new(scroll_hint).style(Style::default().fg(Color::DarkGray).bg(BG_COLOR));
    frame.render_widget(hint_widget, hint_area);
}
