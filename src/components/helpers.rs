use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Creates a centered fixed-size popup area within the given area
pub fn centered_fixed_popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical_layout[1]);

    horizontal_layout[1]
}
