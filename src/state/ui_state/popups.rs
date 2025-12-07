//! Popup state types for the LazyLora TUI.
//!
//! This module contains all popup/modal-related state types including:
//! - Network selection popup state
//! - Search popup state
//! - Network form state for adding custom networks

use super::SearchType;

// ============================================================================
// Network Form
// ============================================================================

/// Fields available in the custom network form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFormField {
    Name,
    AlgodUrl,
    AlgodPort,
    AlgodToken,
    IndexerUrl,
    IndexerPort,
    IndexerToken,
}

impl NetworkFormField {
    #[must_use]
    const fn next(self) -> Self {
        match self {
            Self::Name => Self::AlgodUrl,
            Self::AlgodUrl => Self::AlgodPort,
            Self::AlgodPort => Self::AlgodToken,
            Self::AlgodToken => Self::IndexerUrl,
            Self::IndexerUrl => Self::IndexerPort,
            Self::IndexerPort => Self::IndexerToken,
            Self::IndexerToken => Self::Name,
        }
    }

    #[must_use]
    const fn prev(self) -> Self {
        match self {
            Self::Name => Self::IndexerToken,
            Self::AlgodUrl => Self::Name,
            Self::AlgodPort => Self::AlgodUrl,
            Self::AlgodToken => Self::AlgodPort,
            Self::IndexerUrl => Self::AlgodToken,
            Self::IndexerPort => Self::IndexerUrl,
            Self::IndexerToken => Self::IndexerPort,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::AlgodUrl => "Algod URL",
            Self::AlgodPort => "Algod Port",
            Self::AlgodToken => "Algod API Token (optional)",
            Self::IndexerUrl => "Indexer URL",
            Self::IndexerPort => "Indexer Port",
            Self::IndexerToken => "Indexer API Token (optional)",
        }
    }
}

/// State for the add custom network form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFormState {
    pub name: String,
    pub algod_url: String,
    pub algod_port: String,
    pub algod_token: String,
    pub indexer_url: String,
    pub indexer_port: String,
    pub indexer_token: String,
    pub active_field: NetworkFormField,
    pub return_to_index: usize,
}

impl NetworkFormState {
    #[must_use]
    pub fn new(return_to_index: usize) -> Self {
        Self {
            name: String::new(),
            algod_url: String::new(),
            algod_port: "4001".to_string(),
            algod_token: String::new(),
            indexer_url: String::new(),
            indexer_port: "8980".to_string(),
            indexer_token: String::new(),
            active_field: NetworkFormField::Name,
            return_to_index,
        }
    }

    fn current_value_mut(&mut self) -> &mut String {
        match self.active_field {
            NetworkFormField::Name => &mut self.name,
            NetworkFormField::AlgodUrl => &mut self.algod_url,
            NetworkFormField::AlgodPort => &mut self.algod_port,
            NetworkFormField::AlgodToken => &mut self.algod_token,
            NetworkFormField::IndexerUrl => &mut self.indexer_url,
            NetworkFormField::IndexerPort => &mut self.indexer_port,
            NetworkFormField::IndexerToken => &mut self.indexer_token,
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next();
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.prev();
    }

    pub fn push_char(&mut self, c: char) {
        self.current_value_mut().push(c);
    }

    pub fn backspace(&mut self) {
        self.current_value_mut().pop();
    }
}

// ============================================================================
// Popup State
// ============================================================================

/// Represents the current popup/modal state.
///
/// Only one popup can be active at a time. The popup state
/// determines which overlay is displayed and handles popup-specific data.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PopupState {
    /// No popup is displayed.
    #[default]
    None,
    /// Network selection popup with the currently highlighted index.
    NetworkSelect(usize),
    /// Search popup with query text and search type.
    SearchWithType(String, SearchType),
    /// Message/notification popup.
    Message(String),
    /// Search results display with indexed items.
    SearchResults(Vec<(usize, crate::domain::SearchResultItem)>),
    /// Quit confirmation popup.
    ConfirmQuit,
    /// Custom network form popup.
    NetworkForm(NetworkFormState),
}

impl PopupState {
    /// Returns `true` if there is an active popup.
    ///
    /// # Returns
    ///
    /// `true` if any popup is displayed.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the search query and type if in search mode.
    ///
    /// # Returns
    ///
    /// `Some` tuple of query string and search type if in search mode.
    #[must_use]
    pub fn as_search(&self) -> Option<(&str, SearchType)> {
        match self {
            Self::SearchWithType(query, search_type) => Some((query.as_str(), *search_type)),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_state_variants() {
        // None is inactive, all others are active
        assert!(!PopupState::None.is_active());
        assert!(PopupState::NetworkSelect(0).is_active());
        assert!(PopupState::NetworkForm(NetworkFormState::new(0)).is_active());
        assert!(PopupState::SearchWithType(String::new(), SearchType::Transaction).is_active());
        assert!(PopupState::Message("test".to_string()).is_active());

        // as_search accessor returns correct values
        let search = PopupState::SearchWithType("query".to_string(), SearchType::Account);
        let (q, t) = search.as_search().unwrap();
        assert_eq!(q, "query");
        assert_eq!(t, SearchType::Account);
    }

    #[test]
    fn test_network_form_field_navigation() {
        let mut form = NetworkFormState::new(0);
        assert_eq!(form.active_field, NetworkFormField::Name);

        // Navigate forward through all fields
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::AlgodUrl);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::AlgodPort);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::AlgodToken);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::IndexerUrl);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::IndexerPort);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::IndexerToken);
        form.next_field();
        assert_eq!(form.active_field, NetworkFormField::Name); // Wraps around

        // Navigate backward
        form.prev_field();
        assert_eq!(form.active_field, NetworkFormField::IndexerToken);
    }

    #[test]
    fn test_network_form_input() {
        let mut form = NetworkFormState::new(2);
        assert_eq!(form.return_to_index, 2);

        // Type into name field
        form.push_char('T');
        form.push_char('e');
        form.push_char('s');
        form.push_char('t');
        assert_eq!(form.name, "Test");

        // Backspace
        form.backspace();
        assert_eq!(form.name, "Tes");

        // Move to next field and type
        form.next_field();
        form.push_char('h');
        form.push_char('t');
        form.push_char('t');
        form.push_char('p');
        assert_eq!(form.algod_url, "http");

        // Verify default ports
        assert_eq!(form.algod_port, "4001");
        assert_eq!(form.indexer_port, "8980");
    }
}
