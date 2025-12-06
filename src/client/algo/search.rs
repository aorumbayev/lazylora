//! Search functionality for AlgoClient.

use color_eyre::Result;

use super::AlgoClient;
use crate::domain::SearchResultItem;
use crate::state::SearchType;

impl AlgoClient {
    /// Search by query with specified search type
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or entity is not found.
    pub async fn search_by_query(
        &self,
        query: &str,
        search_type: SearchType,
    ) -> Result<Vec<SearchResultItem>> {
        let results = match search_type {
            SearchType::Transaction => {
                let txns = self.search_transaction(query).await?;
                txns.into_iter()
                    .map(|t| SearchResultItem::Transaction(Box::new(t)))
                    .collect()
            }
            SearchType::Account => self
                .search_address(query)
                .await?
                .map(|a| vec![SearchResultItem::Account(a)])
                .unwrap_or_default(),
            SearchType::Block => match self.search_block(query).await? {
                Some(block) => vec![SearchResultItem::Block(block)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Block '{}' not found. Please enter a valid block number.",
                        query
                    ));
                }
            },
            SearchType::Asset => match self.search_asset(query).await? {
                Some(asset) => vec![SearchResultItem::Asset(asset)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Asset '{}' not found. Please enter a valid asset ID.",
                        query
                    ));
                }
            },
            SearchType::Application => match self.search_application(query).await? {
                Some(app) => vec![SearchResultItem::Application(app)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Application '{}' not found. Please enter a valid application ID.",
                        query
                    ));
                }
            },
        };

        Ok(results)
    }

    /// Get search suggestions based on the current query and search type.
    ///
    /// Provides real-time hints and validation feedback as the user types.
    #[must_use]
    pub fn get_search_suggestions(query: &str, search_type: SearchType) -> String {
        let trimmed = query.trim();

        match search_type {
            SearchType::Account => {
                if trimmed.is_empty() {
                    "Enter an Algorand address or NFD name (e.g., alice.algo)".to_string()
                } else if Self::looks_like_nfd_name(trimmed) {
                    // Could be an NFD name
                    if trimmed.ends_with(".algo") {
                        format!("NFD name '{}'. Press Enter to search.", trimmed)
                    } else {
                        format!(
                            "Looks like NFD name '{}'. Press Enter to search (will try {}.algo).",
                            trimmed, trimmed
                        )
                    }
                } else if trimmed.len() < 58 {
                    format!(
                        "Address too short ({} chars). Try an NFD name or 58-char address.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 58 {
                    format!(
                        "Address too long ({} chars). Algorand addresses are 58 characters long.",
                        trimmed.len()
                    )
                } else if !trimmed
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                {
                    "Address contains invalid characters. Use only uppercase letters and numbers."
                        .to_string()
                } else {
                    "Valid address format. Press Enter to search.".to_string()
                }
            }
            SearchType::Transaction => {
                if trimmed.is_empty() {
                    "Enter a transaction ID (typically 52 characters)".to_string()
                } else if trimmed.len() < 40 {
                    format!(
                        "Transaction ID too short ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 60 {
                    format!(
                        "Transaction ID too long ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else {
                    "Valid transaction ID format. Press Enter to search.".to_string()
                }
            }
            SearchType::Block => {
                if trimmed.is_empty() {
                    "Enter a block number (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Block number must be a positive integer".to_string()
                } else {
                    "Valid block number. Press Enter to search.".to_string()
                }
            }
            SearchType::Asset => {
                if trimmed.is_empty() {
                    "Enter an asset ID (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Asset ID must be a positive integer".to_string()
                } else {
                    "Valid asset ID. Press Enter to search.".to_string()
                }
            }
            SearchType::Application => {
                if trimmed.is_empty() {
                    "Enter an application ID (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Application ID must be a positive integer".to_string()
                } else {
                    "Valid application ID. Press Enter to search.".to_string()
                }
            }
        }
    }
}
