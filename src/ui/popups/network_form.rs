//! Custom network form popup.
//!
//! This popup lets users add a custom network by filling in name, indexer, algod,
//! and optional NFD URL using built-in Ratatui widgets.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::state::ui_state::{NetworkFormField, NetworkFormState};
use crate::theme::{MUTED_COLOR, PRIMARY_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

/// Render the add custom network form.
pub fn render(frame: &mut Frame, area: Rect, form: &NetworkFormState) {
    // Height calculation: 7 fields × 3 lines + 2 lines for help + 2 for popup border = 25
    let popup_area = centered_popup_area(area, 64, 25);
    let popup_block = create_popup_block("Add Custom Network");

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner = popup_block.inner(popup_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    render_field(
        frame,
        rows[0],
        NetworkFormField::Name,
        &form.name,
        form.active_field,
    );
    render_field(
        frame,
        rows[1],
        NetworkFormField::AlgodUrl,
        &form.algod_url,
        form.active_field,
    );
    render_field(
        frame,
        rows[2],
        NetworkFormField::AlgodPort,
        &form.algod_port,
        form.active_field,
    );
    render_field(
        frame,
        rows[3],
        NetworkFormField::AlgodToken,
        &form.algod_token,
        form.active_field,
    );
    render_field(
        frame,
        rows[4],
        NetworkFormField::IndexerUrl,
        &form.indexer_url,
        form.active_field,
    );
    render_field(
        frame,
        rows[5],
        NetworkFormField::IndexerPort,
        &form.indexer_port,
        form.active_field,
    );
    render_field(
        frame,
        rows[6],
        NetworkFormField::IndexerToken,
        &form.indexer_token,
        form.active_field,
    );

    let help = Paragraph::new("Enter: Save & Switch  Tab/Down: Next  Up: Prev  Esc: Cancel")
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);
    frame.render_widget(help, rows[7]);
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    field: NetworkFormField,
    value: &str,
    active: NetworkFormField,
) {
    let is_active = field == active;
    let border_style = if is_active {
        Style::default()
            .fg(PRIMARY_COLOR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };

    let display = if value.is_empty() {
        match field {
            NetworkFormField::Name => "<required>",
            NetworkFormField::IndexerUrl => "http(s)://indexer-host",
            NetworkFormField::IndexerPort => "8980",
            NetworkFormField::IndexerToken => "<optional>",
            NetworkFormField::AlgodUrl => "http(s)://algod-host",
            NetworkFormField::AlgodPort => "4001",
            NetworkFormField::AlgodToken => "<optional>",
        }
    } else {
        value
    };

    let content = if is_active {
        format!("{display}_")
    } else {
        display.to_string()
    };

    let paragraph = Paragraph::new(content)
        .style(if value.is_empty() {
            Style::default().fg(MUTED_COLOR)
        } else {
            Style::default()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(field.label()),
        );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_terminal_80x24;
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    #[rstest]
    fn renders_form(test_terminal_80x24: Terminal<TestBackend>) {
        let mut terminal = test_terminal_80x24;
        let form = NetworkFormState::new(0);

        terminal
            .draw(|f| {
                render(f, f.area(), &form);
            })
            .unwrap();

        assert!(!terminal.backend().buffer().area().is_empty());
    }

    #[test]
    fn renders_form_with_typed_content() {
        // This test needs 80x30 (taller) to fit the full form
        let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
        let mut form = NetworkFormState::new(0);

        // Type "Test" into the name field
        form.push_char('T');
        form.push_char('e');
        form.push_char('s');
        form.push_char('t');

        terminal
            .draw(|f| {
                render(f, f.area(), &form);
            })
            .unwrap();

        // Check that "Test" appears in the buffer
        let buffer = terminal.backend().buffer();
        let buffer_str = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        assert!(
            buffer_str.contains("Test"),
            "Buffer should contain 'Test': {:?}",
            buffer_str
        );
    }

    #[test]
    fn renders_form_snapshot() {
        // This test needs 80x30 (taller) to fit the full form
        let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
        let mut form = NetworkFormState::new(0);

        // Type into name field
        form.push_char('M');
        form.push_char('y');
        form.push_char('N');
        form.push_char('e');
        form.push_char('t');

        terminal
            .draw(|f| {
                render(f, f.area(), &form);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }
}
