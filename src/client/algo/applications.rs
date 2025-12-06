//! Application fetching methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;

use super::AlgoClient;
use crate::domain::{AlgoError, AppStateValue, ApplicationDetails, ApplicationInfo};

impl AlgoClient {
    /// Search for an application by ID.
    pub(crate) async fn search_application(
        &self,
        app_id_str: &str,
    ) -> Result<Option<ApplicationInfo>> {
        let app_id = app_id_str.parse::<u64>().map_err(|_| {
            AlgoError::invalid_input(format!(
                "Invalid application ID '{}'. Please enter a valid positive integer.",
                app_id_str
            ))
            .into_report()
        })?;

        let app_url = format!("{}/v2/applications/{}", self.indexer_url, app_id);

        let response = self.build_indexer_request(&app_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch application #{}: HTTP {} - {}",
                    app_id,
                    status,
                    error_text
                ));
            }
        }

        let app_data: Value = response.json().await?;
        let app = &app_data["application"];

        let creator = app["params"]["creator"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let deleted = app["deleted"].as_bool().unwrap_or(false);

        Ok(Some(ApplicationInfo {
            app_id,
            creator,
            deleted,
        }))
    }

    /// Get detailed application information from indexer.
    ///
    /// # Errors
    ///
    /// Returns an error if the application doesn't exist or network request fails.
    pub async fn get_application_details(&self, app_id: u64) -> Result<ApplicationDetails> {
        let app_url = format!("{}/v2/applications/{}", self.indexer_url, app_id);
        let response = self.build_indexer_request(&app_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AlgoError::not_found("application", app_id.to_string()).into_report());
            }
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch application details: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let app_data: Value = response.json().await?;
        Ok(Self::parse_application_details(&app_data, app_id))
    }

    /// Get basic application info for search results.
    #[allow(dead_code)] // Public API for future use
    pub async fn get_application_info(&self, app_id: u64) -> Result<Option<ApplicationInfo>> {
        let app_url = format!("{}/v2/applications/{}", self.indexer_url, app_id);
        let response = self.build_indexer_request(&app_url).send().await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Ok(None);
            }
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch application: HTTP {}",
                response.status()
            ));
        }

        let app_data: Value = response.json().await?;
        let app = &app_data["application"];

        Ok(Some(ApplicationInfo {
            app_id,
            creator: app["params"]["creator"].as_str().unwrap_or("").to_string(),
            deleted: app["deleted"].as_bool().unwrap_or(false),
        }))
    }

    #[must_use]
    fn parse_application_details(data: &Value, app_id: u64) -> ApplicationDetails {
        let app = &data["application"];
        let params = &app["params"];

        // Parse global state schema
        let global_schema = &params["global-state-schema"];
        let local_schema = &params["local-state-schema"];

        // Parse global state key-value pairs
        let global_state = Self::parse_global_state(&params["global-state"]);

        // Compute application address from app_id
        let app_address = Self::compute_application_address(app_id);

        ApplicationDetails {
            app_id,
            creator: params["creator"].as_str().unwrap_or("").to_string(),
            app_address,
            deleted: app["deleted"].as_bool().unwrap_or(false),
            global_state_byte: global_schema["num-byte-slice"].as_u64().unwrap_or(0),
            global_state_uint: global_schema["num-uint"].as_u64().unwrap_or(0),
            local_state_byte: local_schema["num-byte-slice"].as_u64().unwrap_or(0),
            local_state_uint: local_schema["num-uint"].as_u64().unwrap_or(0),
            extra_program_pages: params["extra-program-pages"].as_u64(),
            approval_program: params["approval-program"].as_str().map(String::from),
            clear_state_program: params["clear-state-program"].as_str().map(String::from),
            global_state,
            created_at_round: app["created-at-round"].as_u64(),
        }
    }

    #[must_use]
    fn parse_global_state(state_json: &Value) -> Vec<AppStateValue> {
        let Some(state_array) = state_json.as_array() else {
            return Vec::new();
        };

        state_array
            .iter()
            .filter_map(|entry| {
                let key_b64 = entry["key"].as_str()?;
                let key = Self::decode_base64_to_string(key_b64);
                let value = &entry["value"];

                let (value_type, decoded_value) = if value["type"].as_u64() == Some(1) {
                    // Bytes type
                    let bytes_b64 = value["bytes"].as_str().unwrap_or("");
                    let decoded = Self::decode_base64_to_string(bytes_b64);
                    // Display as-is if it's printable, otherwise show truncated hex
                    let display_value = if decoded.chars().all(|c| c.is_ascii_graphic() || c == ' ')
                    {
                        decoded
                    } else {
                        // Show truncated hex for binary data
                        let hex = bytes_b64.chars().take(20).collect::<String>();
                        if bytes_b64.len() > 20 {
                            format!("{}...", hex)
                        } else {
                            hex
                        }
                    };
                    ("Bytes".to_string(), display_value)
                } else {
                    // Uint type
                    let uint_val = value["uint"].as_u64().unwrap_or(0);
                    ("Uint".to_string(), uint_val.to_string())
                };

                Some(AppStateValue {
                    key,
                    value_type,
                    value: decoded_value,
                })
            })
            .collect()
    }

    /// Decode Base64 to UTF-8 string, falling back to raw bytes display.
    #[must_use]
    fn decode_base64_to_string(b64: &str) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        match engine.decode(b64) {
            Ok(bytes) => {
                let len = bytes.len();
                String::from_utf8(bytes).unwrap_or_else(|_| format!("<binary:{len} bytes>"))
            }
            Err(_) => b64.to_string(),
        }
    }

    /// Compute the application address from app ID.
    ///
    /// Application addresses are derived by hashing "appID" prefix + app_id bytes.
    #[must_use]
    fn compute_application_address(app_id: u64) -> String {
        use sha2::{Digest, Sha512_256};

        // "appID" prefix as bytes
        let prefix = b"appID";
        let mut hasher = Sha512_256::new();
        hasher.update(prefix);
        hasher.update(app_id.to_be_bytes());
        let hash = hasher.finalize();

        // Encode as base32 (Algorand address format)
        Self::encode_address(&hash)
    }

    /// Encode 32-byte hash as Algorand address (base32 with checksum).
    #[must_use]
    fn encode_address(public_key: &[u8]) -> String {
        use sha2::{Digest, Sha512_256};

        // Compute checksum (last 4 bytes of hash of public key)
        let mut hasher = Sha512_256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        let checksum = &hash[28..32];

        // Concatenate public key + checksum
        let mut addr_bytes = Vec::with_capacity(36);
        addr_bytes.extend_from_slice(public_key);
        addr_bytes.extend_from_slice(checksum);

        // Encode as base32 (no padding)
        data_encoding::BASE32_NOPAD.encode(&addr_bytes)
    }
}
