//! Search popup rendering with type selection.
//!
//! This module provides the search popup that allows users to enter search queries
//! and select the type of entity to search for (Transaction, Block, Account, Asset).

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::client::AlgoClient;
use crate::state::SearchType;
use crate::theme::{BORDER_STYLE, MUTED_COLOR, PRIMARY_COLOR, SUCCESS_COLOR, WARNING_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Public API
// ============================================================================

/// Renders the search popup with type selection.
///
/// Displays a modal search interface where users can:
/// - Enter a search query
/// - Select the type of entity to search for (Transaction, Block, Account, Asset)
/// - See validation suggestions based on the current query and type
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `query` - The current search query text
/// * `search_type` - The currently selected search type
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::search;
/// use lazylora::state::SearchType;
///
/// search::render(&mut frame, area, "ALGO", SearchType::Transaction);
/// ```
pub fn render(frame: &mut Frame, area: Rect, query: &str, search_type: SearchType) {
    let popup_area = centered_popup_area(area, 65, 20);

    let popup_block = create_popup_block("Search Algorand Network");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(BORDER_STYLE)
        .title(" Enter search term ")
        .title_alignment(Alignment::Left);

    let input_area = Rect::new(inner_area.x + 2, inner_area.y + 2, inner_area.width - 4, 3);

    frame.render_widget(input_block.clone(), input_area);

    let text_input_area = input_block.inner(input_area);

    let input_text = format!("{}{}", query, "▏");

    let input = Paragraph::new(input_text)
        .style(Style::default())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(input, text_input_area);

    let selector_y = input_area.y + 4;
    let selector_height = 1;
    let selector_width = inner_area.width / 5;
    let spacing = 2;

    let search_types = [
        SearchType::Transaction,
        SearchType::Block,
        SearchType::Account,
        SearchType::Asset,
    ];

    let mut x_offset = inner_area.x + (inner_area.width - (4 * selector_width + 3 * spacing)) / 2;

    for t in &search_types {
        let is_selected = *t == search_type;
        let button_style = if is_selected {
            Style::default()
                .bg(PRIMARY_COLOR)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };

        let button_rect = Rect::new(x_offset, selector_y, selector_width, selector_height);

        let button = Paragraph::new(t.as_str())
            .style(button_style)
            .alignment(Alignment::Center);

        frame.render_widget(button, button_rect);

        x_offset += selector_width + spacing;
    }

    let suggestions_y = selector_y + 3;
    let suggestions_area = Rect::new(inner_area.x + 2, suggestions_y, inner_area.width - 4, 4);

    let suggestion = AlgoClient::get_search_suggestions(query, search_type);

    let suggestion_color = if suggestion.contains("Valid") {
        SUCCESS_COLOR
    } else if suggestion.contains("too short")
        || suggestion.contains("too long")
        || suggestion.contains("invalid")
    {
        WARNING_COLOR
    } else if suggestion.contains("Enter") {
        MUTED_COLOR
    } else {
        PRIMARY_COLOR
    };

    let suggestions_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(MUTED_COLOR))
        .title(" Suggestions ")
        .title_alignment(Alignment::Left);

    frame.render_widget(suggestions_block.clone(), suggestions_area);

    let suggestions_inner = suggestions_block.inner(suggestions_area);

    let suggestion_text = Paragraph::new(suggestion)
        .style(Style::default().fg(suggestion_color))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(suggestion_text, suggestions_inner);

    let help_text1 = "Search directly queries the Algorand network";
    let help_text2 = "Use Tab to switch between search types";

    let help_area1 = Rect::new(inner_area.x + 2, suggestions_y + 5, inner_area.width - 4, 1);
    let help_area2 = Rect::new(inner_area.x + 2, suggestions_y + 6, inner_area.width - 4, 1);

    let help_msg1 = Paragraph::new(help_text1)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    let help_msg2 = Paragraph::new(help_text2)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg1, help_area1);
    frame.render_widget(help_msg2, help_area2);

    let control_text = "Tab: Change Type  Enter: Search  Esc: Cancel";
    let control_area = Rect::new(
        popup_area.x + (popup_area.width - control_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        control_text.len() as u16,
        1,
    );

    let control_msg = Paragraph::new(control_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(control_msg, control_area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_search_popup_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), "", SearchType::Transaction);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_search_popup_with_query() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), "ALGO123", SearchType::Transaction);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_search_popup_all_types() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test all search types
        for search_type in &[
            SearchType::Transaction,
            SearchType::Block,
            SearchType::Account,
            SearchType::Asset,
        ] {
            terminal
                .draw(|frame| {
                    render(frame, frame.area(), "test", *search_type);
                })
                .unwrap();
        }

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_search_popup_long_query() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let long_query = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";

        terminal
            .draw(|frame| {
                render(frame, frame.area(), long_query, SearchType::Account);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
