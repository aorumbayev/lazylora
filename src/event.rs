use crate::algorand::{AlgoBlock, Network, SearchResultItem, Transaction};
use color_eyre::Result;

/// Events related to network operations and data fetching.
#[derive(Debug)]
pub enum NetworkUpdateEvent {
    StatusUpdate(Result<(), String>),
    BlocksFetched(Result<Vec<AlgoBlock>, String>),
    TransactionsFetched(Result<Vec<Transaction>, String>),
    SearchResults(Result<Vec<SearchResultItem>, String>),
}

/// Application actions triggered by user input or network events.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Action {
    Quit,
    ToggleLiveUpdates,
    RefreshData,
    SwitchFocus,
    MoveSelectionUp,
    MoveSelectionDown,
    ShowDetails,
    CloseDetailsOrPopup,
    OpenNetworkSelector,
    SelectNetworkOption(usize),
    SwitchToNetwork(Network),
    OpenAddCustomNetwork,
    AddCustomNetworkInput(char, usize),
    AddCustomNetworkBackspace(usize),
    AddCustomNetworkFocusNext,
    AddCustomNetworkFocusPrev,
    SaveCustomNetwork {
        name: String,
        algod_url: String,
        indexer_url: String,
        algod_token: Option<String>,
    },
    OpenSearchPopup,
    SearchInput(char),
    SearchBackspace,
    SearchSwitchType,
    PerformSearch(String, crate::app::SearchType),
    SearchResultSelectNext,
    SearchResultSelectPrev,
    SearchResultShowSelected,
    CopySelectedTxnId,
    HandleScrollUp,
    HandleScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ShowMessage(String),
    ClearPopup,

    UpdateNetworkStatus(Result<(), String>),
    UpdateBlocks(Result<Vec<AlgoBlock>, String>),
    UpdateTransactions(Result<Vec<Transaction>, String>),
    UpdateSearchResults(Result<Vec<SearchResultItem>, String>),
    HandleNetworkError(String),
}
