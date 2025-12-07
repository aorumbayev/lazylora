//! Table view rendering for transaction details.
//!
//! Handles the structured key-value table display of transaction information
//! with support for all Algorand transaction types.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Row, Table},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::domain::{Transaction, TransactionDetails};
use crate::state::App;
use crate::theme::{MUTED_COLOR, PRIMARY_COLOR, WARNING_COLOR};

// ============================================================================
// Types
// ============================================================================

/// Represents a row in the flat transaction details list.
#[derive(Debug, Clone)]
pub enum DetailRow {
    /// Key-value info row
    Info { label: String, value: String },
    /// Section header with count
    SectionHeader { title: String, count: usize },
}

// ============================================================================
// Helper Functions
// ============================================================================

fn decode_note_for_display(note: &str) -> String {
    // Try to decode as Base64
    if let Ok(decoded) = BASE64.decode(note) {
        // Try to convert to UTF-8 string
        if let Ok(utf8) = String::from_utf8(decoded) {
            // Successfully decoded - truncate if too long
            let cleaned = utf8.replace('\n', " ").replace('\r', "");
            if cleaned.len() > 50 {
                format!("{}...", &cleaned[..47])
            } else {
                cleaned
            }
        } else {
            // Binary data - show as truncated Base64
            if note.len() > 30 {
                format!("{}... (Base64)", &note[..27])
            } else {
                format!("{} (Base64)", note)
            }
        }
    } else {
        // Not valid Base64 - show as-is (truncated if needed)
        if note.len() > 50 {
            format!("{}...", &note[..47])
        } else {
            note.to_string()
        }
    }
}

// ============================================================================
// Table Mode Rendering
// ============================================================================

/// Renders the transaction details in table mode.
pub fn render_table_mode(txn: &Transaction, app: &App, frame: &mut Frame, area: Rect) {
    // Build flat list of all rows
    let all_rows = build_flat_row_list(txn);

    // Initialize selection if needed
    let selected_row = app.nav.detail_row_index.unwrap_or(0);

    // Build table rows with proper styling
    let rows: Vec<Row> = all_rows
        .iter()
        .enumerate()
        .map(|(idx, detail_row)| {
            let is_selected = selected_row == idx;

            match detail_row {
                DetailRow::Info { label, value } => {
                    let label_style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(PRIMARY_COLOR)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(WARNING_COLOR)
                            .add_modifier(Modifier::BOLD)
                    };
                    let value_style = if is_selected {
                        Style::default().fg(Color::Black).bg(PRIMARY_COLOR)
                    } else {
                        Style::default().fg(PRIMARY_COLOR)
                    };

                    Row::new(vec![
                        Cell::from(label.as_str()).style(label_style),
                        Cell::from(value.as_str()).style(value_style),
                    ])
                }
                DetailRow::SectionHeader { title, count } => {
                    // Create centered section header with decorative lines
                    let header_text = format!("─── {} ({}) ───", title, count);
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(PRIMARY_COLOR)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(MUTED_COLOR)
                    };

                    Row::new(vec![
                        Cell::from("").style(style),
                        Cell::from(header_text).style(style),
                    ])
                }
            }
        })
        .collect();

    // Apply scroll offset
    let scroll_offset = app.nav.detail_row_scroll as usize;
    let visible_rows: Vec<Row> = rows.into_iter().skip(scroll_offset).collect();

    let table = Table::new(visible_rows, [Constraint::Length(18), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, area);
}

// ============================================================================
// Row Building
// ============================================================================

/// Builds the flat list of rows for the table view.
#[must_use]
pub fn build_flat_row_list(txn: &Transaction) -> Vec<DetailRow> {
    let mut rows = Vec::new();

    // Add all basic info rows
    let info_details = build_info_details(txn);
    for (label, value) in info_details {
        rows.push(DetailRow::Info { label, value });
    }

    // Add app call specific sections if present
    if let TransactionDetails::AppCall(app_details) = &txn.details {
        // App Args section
        if !app_details.app_args.is_empty() {
            rows.push(DetailRow::SectionHeader {
                title: "App Args".to_string(),
                count: app_details.app_args.len(),
            });

            for (idx, arg) in app_details.app_args.iter().enumerate() {
                rows.push(DetailRow::Info {
                    label: format!("[{}]:", idx),
                    value: arg.clone(), // Full Base64 value
                });
            }
        }

        // Accounts section
        if !app_details.accounts.is_empty() {
            rows.push(DetailRow::SectionHeader {
                title: "Accounts".to_string(),
                count: app_details.accounts.len(),
            });

            for (idx, account) in app_details.accounts.iter().enumerate() {
                rows.push(DetailRow::Info {
                    label: format!("[{}]:", idx),
                    value: account.clone(), // Full 58-char address
                });
            }
        }

        // Foreign Apps section
        if !app_details.foreign_apps.is_empty() {
            rows.push(DetailRow::SectionHeader {
                title: "Foreign Apps".to_string(),
                count: app_details.foreign_apps.len(),
            });

            for (idx, app_id) in app_details.foreign_apps.iter().enumerate() {
                rows.push(DetailRow::Info {
                    label: format!("[{}]:", idx),
                    value: format!("{}", app_id),
                });
            }
        }

        // Foreign Assets section
        if !app_details.foreign_assets.is_empty() {
            rows.push(DetailRow::SectionHeader {
                title: "Foreign Assets".to_string(),
                count: app_details.foreign_assets.len(),
            });

            for (idx, asset_id) in app_details.foreign_assets.iter().enumerate() {
                rows.push(DetailRow::Info {
                    label: format!("[{}]:", idx),
                    value: format!("{}", asset_id),
                });
            }
        }
    }

    rows
}

/// Builds the transaction details as key-value pairs for the Info tab.
///
/// Public for use by both rendering and copy functionality.
#[must_use]
pub fn build_info_details(txn: &Transaction) -> Vec<(String, String)> {
    let formatted_fee = format!("{:.6} Algos", txn.fee as f64 / 1_000_000.0);

    let mut details: Vec<(String, String)> = vec![
        ("Transaction ID:".to_string(), txn.id.clone()),
        ("Type:".to_string(), txn.txn_type.as_str().to_string()),
        ("From:".to_string(), txn.from.clone()),
    ];

    // Add Group ID if present (shown early as it's important context)
    if let Some(ref group) = txn.group {
        // Truncate Base64 group ID for display
        let group_display = if group.len() > 44 {
            format!("{}...", &group[..44])
        } else {
            group.clone()
        };
        details.push(("Group:".to_string(), group_display));
    }

    // Add Rekey To if present (important security info)
    if let Some(ref rekey_to) = txn.rekey_to {
        details.push(("Rekey To:".to_string(), rekey_to.clone()));
    }

    // Add type-specific fields
    match &txn.details {
        TransactionDetails::Payment(pay_details) => {
            render_payment_details(&mut details, txn, &formatted_fee, pay_details);
        }
        TransactionDetails::AssetTransfer(axfer_details) => {
            render_asset_transfer_details(&mut details, txn, &formatted_fee, axfer_details);
        }
        TransactionDetails::AssetConfig(acfg_details) => {
            render_asset_config_details(&mut details, txn, &formatted_fee, acfg_details);
        }
        TransactionDetails::AssetFreeze(afrz_details) => {
            render_asset_freeze_details(&mut details, txn, &formatted_fee, afrz_details);
        }
        TransactionDetails::AppCall(app_details) => {
            render_app_call_info(&mut details, txn, &formatted_fee, app_details);
        }
        TransactionDetails::KeyReg(keyreg_details) => {
            render_keyreg_details(&mut details, txn, &formatted_fee, keyreg_details);
        }
        TransactionDetails::StateProof(sp_details) => {
            render_state_proof_details(&mut details, txn, &formatted_fee, sp_details);
        }
        TransactionDetails::Heartbeat(hb_details) => {
            render_heartbeat_details(&mut details, txn, &formatted_fee, hb_details);
        }
        TransactionDetails::None => {
            render_default_details(&mut details, txn, &formatted_fee);
        }
    }

    // Add inner transaction count if any
    if !txn.inner_transactions.is_empty() {
        details.push((
            "Inner Txns:".to_string(),
            format!("{}", txn.inner_transactions.len()),
        ));
    }

    // Add note if present and not "None"
    if txn.note != "None" && !txn.note.is_empty() {
        // Try to decode Base64 to UTF-8 for display
        let note_display = decode_note_for_display(&txn.note);
        details.push(("Note:".to_string(), note_display));
    }

    details
}

/// Builds a flat list of all detail rows as (label, value) pairs for copy functionality.
///
/// Similar to `build_flat_row_list` but returns tuples for easier copying.
#[must_use]
pub fn build_flat_row_list_for_copy(txn: &Transaction) -> Vec<(String, String)> {
    let flat_rows = build_flat_row_list(txn);
    flat_rows
        .into_iter()
        .map(|row| match row {
            DetailRow::Info { label, value } => (label, value),
            DetailRow::SectionHeader { title, count } => {
                (format!("─── {} ({}) ───", title, count), String::new())
            }
        })
        .collect()
}

/// Returns the total count of rows in the flat transaction details list.
///
/// Used for navigation bounds checking.
#[must_use]
pub fn get_flat_row_count(txn: &Transaction) -> usize {
    build_flat_row_list(txn).len()
}

// ============================================================================
// Transaction Type-Specific Rendering Functions
// ============================================================================

fn render_payment_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    pay_details: &crate::domain::PaymentDetails,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push((
        "Amount:".to_string(),
        format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0),
    ));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if let Some(close_to) = &pay_details.close_remainder_to {
        details.push(("Close To:".to_string(), close_to.clone()));
    }
    if let Some(close_amount) = pay_details.close_amount {
        details.push((
            "Close Amount:".to_string(),
            format!("{:.6} Algos", close_amount as f64 / 1_000_000.0),
        ));
    }
}

fn render_asset_transfer_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    axfer_details: &crate::domain::AssetTransferDetails,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push(("Amount:".to_string(), format!("{} units", txn.amount)));
    if let Some(asset_id) = txn.asset_id {
        details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
    }
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if let Some(asset_sender) = &axfer_details.asset_sender {
        details.push(("Clawback From:".to_string(), asset_sender.clone()));
    }
    if let Some(close_to) = &axfer_details.close_to {
        details.push(("Close To:".to_string(), close_to.clone()));
    }
    if let Some(close_amount) = axfer_details.close_amount {
        details.push((
            "Close Amount:".to_string(),
            format!("{} units", close_amount),
        ));
    }
}

fn render_asset_config_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    acfg_details: &crate::domain::AssetConfigDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    // Determine if creation or modification
    if let Some(created_id) = acfg_details.created_asset_id {
        details.push(("Created Asset ID:".to_string(), format!("{}", created_id)));
    } else if let Some(asset_id) = acfg_details.asset_id {
        details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
    }

    if let Some(name) = &acfg_details.asset_name {
        details.push(("Asset Name:".to_string(), name.clone()));
    }
    if let Some(unit) = &acfg_details.unit_name {
        details.push(("Unit Name:".to_string(), unit.clone()));
    }
    if let Some(total) = acfg_details.total {
        details.push(("Total Supply:".to_string(), format!("{}", total)));
    }
    if let Some(decimals) = acfg_details.decimals {
        details.push(("Decimals:".to_string(), format!("{}", decimals)));
    }
    if let Some(url) = &acfg_details.url {
        details.push(("URL:".to_string(), url.clone()));
    }
    if let Some(manager) = &acfg_details.manager {
        details.push(("Manager:".to_string(), manager.clone()));
    }
    if let Some(reserve) = &acfg_details.reserve {
        details.push(("Reserve:".to_string(), reserve.clone()));
    }
    if let Some(freeze) = &acfg_details.freeze {
        details.push(("Freeze:".to_string(), freeze.clone()));
    }
    if let Some(clawback) = &acfg_details.clawback {
        details.push(("Clawback:".to_string(), clawback.clone()));
    }
}

fn render_asset_freeze_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    afrz_details: &crate::domain::AssetFreezeDetails,
) {
    details.push((
        "Freeze Target:".to_string(),
        afrz_details.freeze_target.clone(),
    ));
    if let Some(asset_id) = txn.asset_id {
        details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
    }
    details.push((
        "Frozen:".to_string(),
        if afrz_details.frozen { "Yes" } else { "No" }.to_string(),
    ));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}

fn render_app_call_info(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    app_details: &crate::domain::AppCallDetails,
) {
    let app_id_str = if app_details.app_id == 0 {
        "Creating...".to_string()
    } else {
        format!("{}", app_details.app_id)
    };
    details.push(("App ID:".to_string(), app_id_str));
    details.push((
        "On-Complete:".to_string(),
        app_details.on_complete.as_str().to_string(),
    ));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if let Some(created_id) = app_details.created_app_id {
        details.push(("Created App ID:".to_string(), format!("{}", created_id)));
    }

    if !app_details.boxes.is_empty() {
        details.push((
            "Box Refs:".to_string(),
            format!("{} item(s)", app_details.boxes.len()),
        ));
    }
}

fn render_keyreg_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    keyreg_details: &crate::domain::KeyRegDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if keyreg_details.non_participation {
        details.push(("Status:".to_string(), "Going Offline".to_string()));
    } else if keyreg_details.vote_key.is_some() {
        details.push(("Status:".to_string(), "Going Online".to_string()));
        if let Some(vote_key) = &keyreg_details.vote_key {
            let truncated = if vote_key.len() > 20 {
                format!("{}...", &vote_key[..20])
            } else {
                vote_key.clone()
            };
            details.push(("Vote Key:".to_string(), truncated));
        }
        if let Some(sel_key) = &keyreg_details.selection_key {
            let truncated = if sel_key.len() > 20 {
                format!("{}...", &sel_key[..20])
            } else {
                sel_key.clone()
            };
            details.push(("Selection Key:".to_string(), truncated));
        }
        if let Some(vote_first) = keyreg_details.vote_first {
            details.push(("Vote First:".to_string(), format!("{}", vote_first)));
        }
        if let Some(vote_last) = keyreg_details.vote_last {
            details.push(("Vote Last:".to_string(), format!("{}", vote_last)));
        }
        if let Some(dilution) = keyreg_details.vote_key_dilution {
            details.push(("Key Dilution:".to_string(), format!("{}", dilution)));
        }
    }
}

fn render_state_proof_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    sp_details: &crate::domain::StateProofDetails,
) {
    if let Some(sp_type) = sp_details.state_proof_type {
        details.push(("State Proof Type:".to_string(), format!("{}", sp_type)));
    }
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}

fn render_heartbeat_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    hb_details: &crate::domain::HeartbeatDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if let Some(hb_addr) = &hb_details.hb_address {
        details.push(("Heartbeat Addr:".to_string(), hb_addr.clone()));
    }
}

fn render_default_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push(("Amount:".to_string(), format!("{}", txn.amount)));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}
