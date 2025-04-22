use crate::{
    app::{AddCustomNetworkState, App, Focus, PopupState, SearchResultsState, SearchType},
    event::Action,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};

/// Handles a crossterm event and returns an optional Action.
pub fn handle_event(app: &mut App, event: Event) -> Option<Action> {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            return handle_key_press(key, app);
        }
    } else if let Event::Mouse(mouse) = event {
        return handle_mouse_events(mouse, app);
    }
    None
}

/// Handles key press events.
fn handle_key_press(key_event: KeyEvent, app: &mut App) -> Option<Action> {
    if key_event.code == KeyCode::Char('q') {
        return Some(Action::Quit);
    }
    if key_event.code == KeyCode::Char('r') {
        return Some(Action::RefreshData);
    }
    if key_event.code == KeyCode::Char('n') {
        return Some(Action::OpenNetworkSelector);
    }
    if key_event.code == KeyCode::Char('f') {
        return Some(Action::OpenSearchPopup);
    }
    if key_event.code == KeyCode::Char(' ') {
        return Some(Action::ToggleLiveUpdates);
    }

    if (app.show_block_details || app.show_transaction_details) && key_event.code == KeyCode::Esc {
        return Some(Action::CloseDetailsOrPopup);
    }

    if app.popup_state != PopupState::None {
        handle_popup_keys(key_event, app)
    } else {
        handle_main_view_keys(key_event, app)
    }
}

/// Handles key events when the network selection popup is active.
fn handle_network_selector_keys(key_event: KeyEvent, app: &mut App) -> Option<Action> {
    if let PopupState::NetworkSelect {
        selected_index,
        available_networks,
    } = &mut app.popup_state
    {
        let num_options = available_networks.len() + 1;
        match key_event.code {
            KeyCode::Esc => Some(Action::ClearPopup),
            KeyCode::Up => {
                let new_index = selected_index.saturating_sub(1);
                Some(Action::SelectNetworkOption(new_index))
            }
            KeyCode::Down => {
                let new_index = (*selected_index + 1) % num_options;
                Some(Action::SelectNetworkOption(new_index))
            }
            KeyCode::Enter => {
                if *selected_index < available_networks.len() {
                    let network_to_switch = available_networks[*selected_index].clone();
                    Some(Action::SwitchToNetwork(network_to_switch))
                } else {
                    Some(Action::OpenAddCustomNetwork)
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Handles key events when the "Add Custom Network" popup is active.
fn handle_add_custom_network_keys(
    key_event: KeyEvent,
    state: &mut AddCustomNetworkState,
) -> Option<Action> {
    match key_event.code {
        KeyCode::Esc => Some(Action::ClearPopup),
        KeyCode::Tab => Some(Action::AddCustomNetworkFocusNext),
        KeyCode::BackTab => Some(Action::AddCustomNetworkFocusPrev),
        KeyCode::Char(c) => Some(Action::AddCustomNetworkInput(
            c,
            state.focused_field.as_index(),
        )),
        KeyCode::Backspace => Some(Action::AddCustomNetworkBackspace(
            state.focused_field.as_index(),
        )),
        KeyCode::Enter => {
            let final_token = if state.algod_token.trim().is_empty() {
                None
            } else {
                Some(state.algod_token.clone())
            };
            Some(Action::SaveCustomNetwork {
                name: state.name.clone(),
                algod_url: state.algod_url.clone(),
                indexer_url: state.indexer_url.clone(),
                algod_token: final_token,
            })
        }
        _ => None,
    }
}

/// Handles key events when the search popup is active.
fn handle_search_with_type_keys(
    key_event: KeyEvent,
    query: &mut String,
    search_type: &mut SearchType,
) -> Option<Action> {
    match key_event.code {
        KeyCode::Esc => Some(Action::ClearPopup),
        KeyCode::Char(c) => Some(Action::SearchInput(c)),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Tab => Some(Action::SearchSwitchType),
        KeyCode::Enter => Some(Action::PerformSearch(query.clone(), *search_type)),
        _ => None,
    }
}

/// Handles key events when the search results popup is active.
fn handle_search_results_keys(
    key_event: KeyEvent,
    _state: &mut SearchResultsState,
) -> Option<Action> {
    match key_event.code {
        KeyCode::Esc => Some(Action::ClearPopup),
        KeyCode::Up => Some(Action::SearchResultSelectPrev),
        KeyCode::Down => Some(Action::SearchResultSelectNext),
        KeyCode::Enter => Some(Action::SearchResultShowSelected),
        _ => None,
    }
}

/// Handles key events when the main view is active (no popups or details).
fn handle_main_view_keys(key_event: KeyEvent, app: &mut App) -> Option<Action> {
    match app.focus {
        Focus::Blocks => handle_block_keys(key_event, app),
        Focus::Transactions => handle_transaction_keys(key_event, app),
    }
}

fn handle_block_keys(key_event: KeyEvent, _app: &mut App) -> Option<Action> {
    match key_event.code {
        KeyCode::Up => Some(Action::MoveSelectionUp),
        KeyCode::Down => Some(Action::MoveSelectionDown),
        KeyCode::PageUp => Some(Action::ScrollPageUp),
        KeyCode::PageDown => Some(Action::ScrollPageDown),
        KeyCode::Enter => Some(Action::ShowDetails),
        KeyCode::Tab => Some(Action::SwitchFocus),
        _ => None,
    }
}

fn handle_transaction_keys(key_event: KeyEvent, _app: &mut App) -> Option<Action> {
    match key_event.code {
        KeyCode::Up => Some(Action::MoveSelectionUp),
        KeyCode::Down => Some(Action::MoveSelectionDown),
        KeyCode::PageUp => Some(Action::ScrollPageUp),
        KeyCode::PageDown => Some(Action::ScrollPageDown),
        KeyCode::Enter => Some(Action::ShowDetails),
        KeyCode::Char('c') => Some(Action::CopySelectedTxnId),
        KeyCode::Tab => Some(Action::SwitchFocus),
        _ => None,
    }
}

/// Handles mouse events.
fn handle_mouse_events(mouse_event: MouseEvent, _app: &mut App) -> Option<Action> {
    match mouse_event.kind {
        MouseEventKind::ScrollDown => Some(Action::HandleScrollDown),
        MouseEventKind::ScrollUp => Some(Action::HandleScrollUp),
        _ => None,
    }
}

pub fn handle_popup_keys(key_event: KeyEvent, app: &mut App) -> Option<Action> {
    match &mut app.popup_state {
        PopupState::NetworkSelect { .. } => handle_network_selector_keys(key_event, app),
        PopupState::AddCustomNetwork(state) => handle_add_custom_network_keys(key_event, state),
        PopupState::SearchWithType { query, search_type } => {
            handle_search_with_type_keys(key_event, query, search_type)
        }
        PopupState::SearchResults(state) => handle_search_results_keys(key_event, state),
        PopupState::Message(_) => {
            if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Enter {
                Some(Action::ClearPopup)
            } else {
                None
            }
        }
        PopupState::None => None,
    }
}
