//! Shared test utilities and Mother pattern factories.
//!
//! This module provides reusable test fixtures following the Mother pattern
//! as described in AGENTS.md. Use these helpers to avoid copy-pasting
//! setup code across tests.
#![allow(dead_code)]

use crate::domain::{
    AlgoBlock, BlockDetails, BlockInfo, Transaction, TransactionDetails, TxnType,
    account::{AccountDetails, AccountInfo, ParticipationInfo},
    asset::{AssetDetails, AssetInfo},
};

// ============================================================================
// Mother Pattern Factories
// ============================================================================

#[allow(dead_code)]
pub struct BlockMother;

impl BlockMother {
    #[must_use]
    pub fn with_id(id: u64) -> AlgoBlock {
        AlgoBlock {
            id,
            txn_count: 5,
            timestamp: "2024-01-01 12:00:00".to_string(),
        }
    }

    #[must_use]
    pub fn with_txn_count(id: u64, txn_count: u16) -> AlgoBlock {
        AlgoBlock {
            id,
            txn_count,
            timestamp: "2024-01-01 12:00:00".to_string(),
        }
    }
}

#[allow(dead_code)]
pub struct TransactionMother;

impl TransactionMother {
    #[must_use]
    pub fn payment(id: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            txn_type: TxnType::Payment,
            from: "sender".to_string(),
            to: "receiver".to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            block: 12345,
            fee: 1000,
            note: String::new(),
            amount: 1_000_000,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    #[must_use]
    pub fn payment_with_addresses(id: &str, from: &str, to: &str) -> Transaction {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            ..Self::payment(id)
        }
    }

    #[must_use]
    pub fn app_call(id: &str, app_id: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            txn_type: TxnType::AppCall,
            from: "caller".to_string(),
            to: app_id.to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            block: 12345,
            fee: 1000,
            note: String::new(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    #[must_use]
    pub fn asset_transfer(id: &str, asset_id: u64, amount: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            txn_type: TxnType::AssetTransfer,
            from: "sender".to_string(),
            to: "receiver".to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            block: 12345,
            fee: 1000,
            note: String::new(),
            amount,
            asset_id: Some(asset_id),
            rekey_to: None,
            group: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_inner(parent: Transaction, inner: Vec<Transaction>) -> Transaction {
        Transaction {
            inner_transactions: inner,
            ..parent
        }
    }

    // ========================================================================
    // Graph Test Fixtures - Static transactions for offline graph tests
    // ========================================================================

    fn base_txn(id: &str, txn_type: TxnType, from: &str, to: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            txn_type,
            from: from.to_string(),
            to: to.to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            block: 12345,
            fee: 1000,
            note: String::new(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    // --- Payment Fixtures ---

    #[must_use]
    pub fn mainnet_payment() -> Transaction {
        use crate::domain::PaymentDetails;
        Transaction {
            amount: 236_070_000, // 236.07 ALGO
            details: TransactionDetails::Payment(PaymentDetails {
                close_remainder_to: None,
                close_amount: None,
            }),
            ..Self::base_txn(
                "FBORGSDC4ULLWHWZUMUFIYQLSDC26HGLTFD7EATQDY37FHCIYBBQ",
                TxnType::Payment,
                "M3IYZPKQ7RDQANC4GZQFOX6R5SGS7KEUMNH5TJDIECKD5WRLZXM",
                "KIZHKFVDNYAQ4QSQKAXFV6I5N6I5XPE2M2LI6LHIIWD72LXWBQ",
            )
        }
    }

    #[must_use]
    pub fn mainnet_payment_close_remainder() -> Transaction {
        use crate::domain::PaymentDetails;
        Transaction {
            amount: 5_000_000,
            details: TransactionDetails::Payment(PaymentDetails {
                close_remainder_to: Some(
                    "CLOSE7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string(),
                ),
                close_amount: Some(10_000_000),
            }),
            ..Self::base_txn(
                "ILDCD5Z64CYSLEZIHBG5DVME2ITJI2DIVZAPDPEWPCYMTRA5SVGA",
                TxnType::Payment,
                "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
            )
        }
    }

    #[must_use]
    pub fn testnet_rekey() -> Transaction {
        use crate::domain::PaymentDetails;
        Transaction {
            amount: 0,
            rekey_to: Some("QUASTY2O6HXBETQPTZOGKSXEODSO7NZQMKWMQQ6JSFMWILNWSA".to_string()),
            details: TransactionDetails::Payment(PaymentDetails {
                close_remainder_to: None,
                close_amount: None,
            }),
            ..Self::base_txn(
                "24RAYAOGMJ45BL6A7RYQOKZNECCA3VFXQUAM5X64BEDBVFNLPIPQ",
                TxnType::Payment,
                "WK3YOMGKDIPB7VGK2TRQPCMKB6FBXCGYV72WKDVF7OZ3GQGTM",
                "WK3YOMGKDIPB7VGK2TRQPCMKB6FBXCGYV72WKDVF7OZ3GQGTM",
            )
        }
    }

    // --- Asset Transfer Fixtures ---

    #[must_use]
    pub fn mainnet_asset_transfer() -> Transaction {
        use crate::domain::AssetTransferDetails;
        Transaction {
            amount: 1_000_000, // 1 USDC
            asset_id: Some(31_566_704),
            details: TransactionDetails::AssetTransfer(AssetTransferDetails {
                asset_sender: None,
                close_to: None,
                close_amount: None,
            }),
            ..Self::base_txn(
                "JBDSQEI37W5KWPQICT2IGCG2FWMUGJEUYYK3KFKNSYRNAXU2ARUA",
                TxnType::AssetTransfer,
                "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
            )
        }
    }

    #[must_use]
    pub fn mainnet_asset_opt_in() -> Transaction {
        use crate::domain::AssetTransferDetails;
        let addr = "OPTIN7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A";
        Transaction {
            amount: 0,
            asset_id: Some(31_566_704),
            details: TransactionDetails::AssetTransfer(AssetTransferDetails {
                asset_sender: None,
                close_to: None,
                close_amount: None,
            }),
            ..Self::base_txn(
                "563MNGEL2OF4IBA7CFLIJNMBETT5QNKZURSLIONJBTJFALGYOAUA",
                TxnType::AssetTransfer,
                addr,
                addr,
            )
        }
    }

    #[must_use]
    pub fn mainnet_asset_close_to() -> Transaction {
        use crate::domain::AssetTransferDetails;
        Transaction {
            amount: 0,
            asset_id: Some(31_566_704),
            details: TransactionDetails::AssetTransfer(AssetTransferDetails {
                asset_sender: None,
                close_to: Some("CLOSETO7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string()),
                close_amount: Some(500_000),
            }),
            ..Self::base_txn(
                "J7AC3HPOSQNKUVYDCNO4UC3XXRR3BVWYWXV6UL3BCZVNODO63LDA",
                TxnType::AssetTransfer,
                "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
            )
        }
    }

    #[must_use]
    pub fn testnet_asset_clawback() -> Transaction {
        use crate::domain::AssetTransferDetails;
        Transaction {
            amount: 100,
            asset_id: Some(12_345_678),
            details: TransactionDetails::AssetTransfer(AssetTransferDetails {
                asset_sender: Some(
                    "CLAWBACK_TARGET7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string(),
                ),
                close_to: None,
                close_amount: None,
            }),
            ..Self::base_txn(
                "VIXTUMAPT7NR4RB2WVOGMETW4QY43KIDA3HWDWWXS3UEDKGTEECQ",
                TxnType::AssetTransfer,
                "CLAWBACK7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
            )
        }
    }

    // --- Asset Config Fixtures ---

    #[must_use]
    pub fn mainnet_asset_config_create() -> Transaction {
        use crate::domain::AssetConfigDetails;
        Transaction {
            details: TransactionDetails::AssetConfig(AssetConfigDetails {
                asset_id: None,
                created_asset_id: Some(987_654_321),
                total: Some(1_000_000_000),
                decimals: Some(6),
                default_frozen: Some(false),
                asset_name: Some("Test Token".to_string()),
                unit_name: Some("TEST".to_string()),
                url: Some("https://test.com".to_string()),
                metadata_hash: None,
                manager: Some("MANAGER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string()),
                reserve: None,
                freeze: None,
                clawback: None,
            }),
            ..Self::base_txn(
                "PJHUAFK4UMBABT2Q24PHG52R63YOOKSHK7XL226PDCIG2Y2PQSFQ",
                TxnType::AssetConfig,
                "CREATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "unknown",
            )
        }
    }

    #[must_use]
    pub fn mainnet_asset_config_reconfigure() -> Transaction {
        use crate::domain::AssetConfigDetails;
        Transaction {
            asset_id: Some(12_345_678),
            details: TransactionDetails::AssetConfig(AssetConfigDetails {
                asset_id: Some(12_345_678),
                created_asset_id: None,
                total: None,
                decimals: None,
                default_frozen: None,
                asset_name: None,
                unit_name: None,
                url: None,
                metadata_hash: None,
                manager: Some("NEWMANAGER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string()),
                reserve: None,
                freeze: None,
                clawback: None,
            }),
            ..Self::base_txn(
                "PBWCKDUCNKTFTYMDCMDSMFJDV7NHJYL2GXNA4GL7RCTZWUKNNPVQ",
                TxnType::AssetConfig,
                "MANAGER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "unknown",
            )
        }
    }

    // --- Asset Freeze Fixtures ---

    #[must_use]
    pub fn mainnet_asset_freeze() -> Transaction {
        use crate::domain::AssetFreezeDetails;
        Transaction {
            asset_id: Some(31_566_704),
            details: TransactionDetails::AssetFreeze(AssetFreezeDetails {
                frozen: true,
                freeze_target: "FROZEN7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A".to_string(),
            }),
            ..Self::base_txn(
                "2XFGVOHMFYLAWBHOSIOI67PBT5LDRHBTD3VLX5EYBDTFNVKMCJIA",
                TxnType::AssetFreeze,
                "FREEZER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "FROZEN7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
            )
        }
    }

    // --- Key Registration Fixtures ---

    #[must_use]
    pub fn mainnet_keyreg_offline() -> Transaction {
        use crate::domain::KeyRegDetails;
        Transaction {
            details: TransactionDetails::KeyReg(KeyRegDetails {
                vote_key: None,
                selection_key: None,
                state_proof_key: None,
                vote_first: None,
                vote_last: None,
                vote_key_dilution: None,
                non_participation: true,
            }),
            ..Self::base_txn(
                "VE767RE4HGQM7GFC7MUVY3J67KOR5TV34OBTDDEQTDET2UFM7KTQ",
                TxnType::KeyReg,
                "VALIDATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "",
            )
        }
    }

    #[must_use]
    pub fn mainnet_keyreg_online() -> Transaction {
        use crate::domain::KeyRegDetails;
        Transaction {
            details: TransactionDetails::KeyReg(KeyRegDetails {
                vote_key: Some("vote_key_base64".to_string()),
                selection_key: Some("selection_key_base64".to_string()),
                state_proof_key: Some("state_proof_key_base64".to_string()),
                vote_first: Some(1000),
                vote_last: Some(3_000_000),
                vote_key_dilution: Some(10_000),
                non_participation: false,
            }),
            ..Self::base_txn(
                "NPJHKQW2XH6EYS6NRCXLMSWVXXNJYWV5UA6DN2HKLQYQXPTVRAZA",
                TxnType::KeyReg,
                "VALIDATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "",
            )
        }
    }

    // --- App Call Fixtures ---

    #[must_use]
    pub fn mainnet_app_call_with_inner_txns() -> Transaction {
        use crate::domain::{AppCallDetails, OnComplete};

        // Inner payment
        let inner_pay = Transaction {
            amount: 2_770_000, // 2.77 ALGO
            ..Self::base_txn(
                "INNER_PAY_1",
                TxnType::Payment,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "GEA4RFJZWDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZQXTU",
            )
        };

        // Inner app call
        let inner_app = Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 1002541853,
                created_app_id: None,
                on_complete: OnComplete::NoOp,
                approval_program: None,
                clear_state_program: None,
                app_args: vec![],
                accounts: vec![],
                foreign_apps: vec![],
                foreign_assets: vec![],
                boxes: vec![],
                global_state_schema: None,
                local_state_schema: None,
                extra_program_pages: None,
            }),
            ..Self::base_txn(
                "INNER_APP_1",
                TxnType::AppCall,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "1002541853",
            )
        };

        // Inner asset transfer (from the app call's inner)
        let inner_axf = Transaction {
            amount: 586_582,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_1",
                TxnType::AssetTransfer,
                "2PIQX3N6TLP5JMHHXMWM3Q3VXVMQG3V5QZQX5QZQX5QZQX5QMM",
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
            )
        };

        // Nested inner within app call
        let inner_app_with_axf = Self::with_inner(inner_app, vec![inner_axf]);

        // More inner transactions for complexity
        let inner_axf_2 = Transaction {
            amount: 586_582,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_2",
                TxnType::AssetTransfer,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "FCH7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZSM",
            )
        };

        let inner_app_2 = Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 1002541853,
                on_complete: OnComplete::NoOp,
                ..Default::default()
            }),
            ..Self::base_txn(
                "INNER_APP_2",
                TxnType::AppCall,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "1002541853",
            )
        };

        let inner_axf_3 = Transaction {
            amount: 1465,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_3",
                TxnType::AssetTransfer,
                "FCH7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZSM",
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
            )
        };

        let inner_app_2_with_axf = Self::with_inner(inner_app_2, vec![inner_axf_3]);

        let inner_axf_4 = Transaction {
            amount: 1465,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_4",
                TxnType::AssetTransfer,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "EOX7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZE4",
            )
        };

        let inner_app_3 = Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 1002541853,
                on_complete: OnComplete::NoOp,
                ..Default::default()
            }),
            ..Self::base_txn(
                "INNER_APP_3",
                TxnType::AppCall,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "1002541853",
            )
        };

        let inner_axf_5 = Transaction {
            amount: 17647,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_5",
                TxnType::AssetTransfer,
                "EOX7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZE4",
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
            )
        };

        let inner_app_3_with_axf = Self::with_inner(inner_app_3, vec![inner_axf_5]);

        let inner_axf_6 = Transaction {
            amount: 17647,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF_6",
                TxnType::AssetTransfer,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "IWT7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZVY",
            )
        };

        let inner_app_4 = Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 1090720165,
                on_complete: OnComplete::NoOp,
                ..Default::default()
            }),
            ..Self::base_txn(
                "INNER_APP_4",
                TxnType::AppCall,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "1090720165",
            )
        };

        let inner_pay_from_app = Transaction {
            amount: 2_790_000, // 2.79 ALGO
            ..Self::base_txn(
                "INNER_PAY_2",
                TxnType::Payment,
                "IWT7GN3ZXDPB5YQVLV3XVLK2QZQX5QZQX5QZQX5QZQX5QZVY",
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
            )
        };

        let inner_app_4_with_pay = Self::with_inner(inner_app_4, vec![inner_pay_from_app]);

        let inner_self_pay = Transaction {
            amount: 0,
            ..Self::base_txn(
                "INNER_SELF_PAY",
                TxnType::Payment,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
            )
        };

        // Outer app call with all inner transactions
        Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 1200000000,
                created_app_id: None,
                on_complete: OnComplete::NoOp,
                approval_program: None,
                clear_state_program: None,
                app_args: vec![],
                accounts: vec![],
                foreign_apps: vec![],
                foreign_assets: vec![],
                boxes: vec![],
                global_state_schema: None,
                local_state_schema: None,
                extra_program_pages: None,
            }),
            inner_transactions: vec![
                inner_pay,
                inner_app_with_axf,
                inner_axf_2,
                inner_app_2_with_axf,
                inner_axf_4,
                inner_app_3_with_axf,
                inner_axf_6,
                inner_app_4_with_pay,
                inner_self_pay,
            ],
            ..Self::base_txn(
                "INDQXWQXHF22SO45EZY7V6FFNI6WUD5FHRVDV6NCU6HD424BJGGA",
                TxnType::AppCall,
                "AAC7XKVWL5YSZDWU2VCWQRDQXN4VMQG3VTSQV6I2XQVYQV7X4A",
                "1200000000",
            )
        }
    }

    #[must_use]
    pub fn mainnet_app_call_mixed_inner() -> Transaction {
        use crate::domain::{AppCallDetails, OnComplete};

        let inner_pay = Transaction {
            amount: 1_000_000,
            ..Self::base_txn(
                "INNER_PAY",
                TxnType::Payment,
                "CALLER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
            )
        };

        let inner_axf = Transaction {
            amount: 500_000,
            asset_id: Some(31_566_704),
            ..Self::base_txn(
                "INNER_AXF",
                TxnType::AssetTransfer,
                "CALLER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "RECEIVER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
            )
        };

        Transaction {
            details: TransactionDetails::AppCall(AppCallDetails {
                app_id: 123_456_789,
                on_complete: OnComplete::NoOp,
                ..Default::default()
            }),
            inner_transactions: vec![inner_pay, inner_axf],
            ..Self::base_txn(
                "IBB54TEAX4WYSD7AUA2EYPHSSXG3VKFVKEKU3363TJUL7JCTFBVQ",
                TxnType::AppCall,
                "CALLER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
                "123456789",
            )
        }
    }

    #[must_use]
    pub fn mainnet_large_app_call() -> Transaction {
        // Reuse the complex inner txn fixture
        Self::mainnet_app_call_with_inner_txns()
    }
}

#[allow(dead_code)]
pub struct AccountMother;

impl AccountMother {
    #[must_use]
    pub fn basic(address: &str, balance: u64) -> AccountInfo {
        AccountInfo {
            address: address.to_string(),
            balance,
            pending_rewards: 0,
            reward_base: 0,
            status: "Offline".to_string(),
            assets_count: 0,
            created_assets_count: 0,
        }
    }

    #[must_use]
    pub fn details(address: &str, balance: u64) -> AccountDetails {
        AccountDetails {
            address: address.to_string(),
            balance,
            min_balance: 100_000,
            pending_rewards: 0,
            rewards: 0,
            reward_base: 0,
            status: "Offline".to_string(),
            total_apps_opted_in: 0,
            total_assets_opted_in: 0,
            total_created_apps: 0,
            total_created_assets: 0,
            total_boxes: 0,
            auth_addr: None,
            participation: None,
            assets: Vec::new(),
            created_assets: Vec::new(),
            apps_local_state: Vec::new(),
            created_apps: Vec::new(),
            nfd: None,
        }
    }

    #[must_use]
    pub fn online(address: &str, vote_first: u64, vote_last: u64) -> AccountDetails {
        AccountDetails {
            status: "Online".to_string(),
            participation: Some(ParticipationInfo {
                vote_first,
                vote_last,
                vote_key_dilution: 10_000,
                selection_key: "c2VsS2V5".to_string(),
                vote_key: "dm90ZUtleQ==".to_string(),
                state_proof_key: Some("c3BLZXk=".to_string()),
            }),
            ..Self::details(address, 10_000_000)
        }
    }

    #[must_use]
    pub fn rekeyed(address: &str, auth_addr: &str) -> AccountDetails {
        AccountDetails {
            auth_addr: Some(auth_addr.to_string()),
            ..Self::details(address, 5_000_000)
        }
    }

    /// Creates an account with NFD (silvio.algo-style).
    ///
    /// This fixture mirrors the silvio.algo mainnet account but with
    /// stable values for snapshot testing.
    #[must_use]
    pub fn with_nfd() -> AccountDetails {
        use crate::domain::account::AccountAssetHolding;
        use crate::domain::nfd::NfdInfo;

        AccountDetails {
            address: "5NBAJP3FDBY4HXY3RZWRBE3VG4YJLXWOULC2QC4WM75KKCX4JZYG4ASVJ4".to_string(),
            balance: 5_100_000,   // 5.1 ALGO (stable for tests)
            min_balance: 200_000, // 0.2 ALGO
            pending_rewards: 0,
            rewards: 0,
            reward_base: 0,
            status: "Offline".to_string(),
            total_apps_opted_in: 2,
            total_assets_opted_in: 3,
            total_created_apps: 0,
            total_created_assets: 0,
            total_boxes: 0,
            auth_addr: None,
            participation: None,
            assets: vec![AccountAssetHolding {
                asset_id: 31_566_704, // USDC
                amount: 1_000_000,
                is_frozen: false,
            }],
            created_assets: Vec::new(),
            apps_local_state: Vec::new(),
            created_apps: Vec::new(),
            nfd: Some(NfdInfo {
                name: "silvio.algo".to_string(),
                deposit_account: Some(
                    "5NBAJP3FDBY4HXY3RZWRBE3VG4YJLXWOULC2QC4WM75KKCX4JZYG4ASVJ4".to_string(),
                ),
                owner: Some(
                    "5NBAJP3FDBY4HXY3RZWRBE3VG4YJLXWOULC2QC4WM75KKCX4JZYG4ASVJ4".to_string(),
                ),
                avatar_url: None,
                is_verified: true,
            }),
        }
    }
}

#[allow(dead_code)]
pub struct AssetMother;

impl AssetMother {
    #[must_use]
    pub fn basic(id: u64, name: &str, unit: &str) -> AssetInfo {
        AssetInfo {
            id,
            name: name.to_string(),
            unit_name: unit.to_string(),
            creator: "CREATOR_ADDRESS".to_string(),
            total: 1_000_000_000_000,
            decimals: 6,
            url: String::new(),
        }
    }

    /// Mainnet USDC (ID: 31566704) - matches real data for snapshot tests.
    #[must_use]
    pub fn usdc() -> AssetDetails {
        AssetDetails {
            id: 31_566_704,
            name: "USDC".to_string(),
            unit_name: "USDC".to_string(),
            creator: "2UEQTE5QDNXPI7M3TU44G6SYKLFWLPQO7EBZM7K7MHMQQAPMT4".to_string(),
            total: 18_446_744_073_709_550_781,
            decimals: 6,
            url: "https://www.centre.io/usdc".to_string(),
            metadata_hash: None,
            default_frozen: false,
            manager: Some("37XL3M57AXBUJARWMT5R7M35OERQXQP3AHXRQ2IQINAHPBOYQFUQTAFGWI".to_string()),
            reserve: Some("2UEQTE5QDNXPI7M3TU44G6SYKLFWLPQO7EBZM7K7MHMQQAPMT4YQDAVLTE".to_string()),
            freeze: Some("3ERES6JFBIJ7ZPNVQJNH2LETCBQU6R4SJJQABD7K4KYLSBXP6KQQK4CDMU".to_string()),
            clawback: Some(
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ".to_string(),
            ),
            deleted: false,
            created_at_round: Some(8_874_561),
        }
    }

    /// Mainnet goUSD (ID: 672913181) - matches real data for snapshot tests.
    #[must_use]
    pub fn gousd() -> AssetDetails {
        AssetDetails {
            id: 672_913_181,
            name: "goUSD".to_string(),
            unit_name: "goUSD".to_string(),
            creator: "PNC3CKZTHOIMGMSG7KPUCF3XA6ILZMBXMD5YVWT7GUSBOQOFV3UDFE73NU".to_string(),
            total: 18_446_744_073_709_550_781,
            decimals: 6,
            url: "https://www.algomint.io".to_string(),
            metadata_hash: Some("NjgxMWRlZGY2ZWJlNGI1MDJkMDkwODVmYzkzMWQzNDM=".to_string()),
            default_frozen: false,
            manager: Some("PXLRHMSOTI5LDPRNMMM5F4NDPLCU6AQRFLEYY7PDXK5DZZW2CKEPQLTQEU".to_string()),
            reserve: Some("NLTFR6Y7AAQ6BFFE7NMNJRK3CIZH5S4XQVVLM3NWRPXRDHC5Y3DDPYFH3U".to_string()),
            freeze: Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ".to_string()),
            clawback: Some(
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ".to_string(),
            ),
            deleted: false,
            created_at_round: Some(19_982_265),
        }
    }

    #[must_use]
    pub fn details(id: u64, name: &str, unit: &str, total: u64, decimals: u64) -> AssetDetails {
        AssetDetails {
            id,
            name: name.to_string(),
            unit_name: unit.to_string(),
            creator: "CREATOR_ADDRESS".to_string(),
            total,
            decimals,
            url: String::new(),
            metadata_hash: None,
            default_frozen: false,
            manager: Some("MANAGER_ADDRESS".to_string()),
            reserve: None,
            freeze: None,
            clawback: None,
            deleted: false,
            created_at_round: None,
        }
    }

    #[must_use]
    pub fn immutable(id: u64, name: &str) -> AssetDetails {
        AssetDetails {
            manager: None,
            reserve: None,
            freeze: None,
            clawback: None,
            ..Self::details(id, name, "TOKEN", 1_000_000, 0)
        }
    }
}

// ============================================================================
// rstest Fixtures
// ============================================================================

use crate::state::{App, StartupOptions};
use ratatui::{Terminal, backend::TestBackend};
use rstest::fixture;

#[fixture]
pub fn test_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(100, 40)).expect("terminal creation should succeed")
}

#[fixture]
pub fn test_terminal_80x24() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(80, 24)).expect("terminal creation should succeed")
}

#[fixture]
pub async fn mock_app() -> App {
    App::new(StartupOptions::default())
        .await
        .expect("app creation should succeed")
}

/// Extended block factory (complements existing BlockMother).
impl BlockMother {
    #[must_use]
    pub fn info(id: u64, txn_count: u16) -> BlockInfo {
        BlockInfo {
            id,
            timestamp: "2024-01-01 12:00:00".to_string(),
            txn_count,
            proposer: "PROPOSER_ADDRESS".to_string(),
            seed: "SEED_VALUE".to_string(),
        }
    }

    #[must_use]
    pub fn details(id: u64, transactions: Vec<Transaction>) -> BlockDetails {
        let info = Self::info(id, transactions.len() as u16);
        BlockDetails::new(info, transactions)
    }

    /// Creates a static fixture matching mainnet block 50,000,000.
    ///
    /// This fixture eliminates network calls in block detail tests.
    /// Data matches the real block for snapshot consistency.
    #[must_use]
    pub fn mainnet_block_50m() -> BlockDetails {
        use std::collections::HashMap;

        let info = BlockInfo {
            id: 50_000_000,
            timestamp: "Sat, 17 May 2025 02:37:15".to_string(),
            txn_count: 32,
            proposer: "4TPMQLUIBMQ6ILR4FSBEWJACEOYJVYZ7PWL333KL47DHVT6TJHH55E5WWE".to_string(),
            seed: "SEED_VALUE".to_string(),
        };

        // Create transactions matching the snapshot order
        let transactions = vec![
            Self::block_txn(
                "LIILHSS7G6YKV2RY2WQN",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "KSAFVKQPUAEZSBZE3FPY",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "E7DNSQZR75N4QOX6YS6K",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "3LIWKI3VBVDKPKERYO2I",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "XAKZHMB4XR4CC6HPYT3Y",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "QVATH6VVIAE5BWONU5B7",
                TxnType::AppCall,
                "6XHBAFTDDSGTD4AOR67S",
            ),
            Self::block_txn(
                "HZY4SIHEHEUD4HN7FK4J",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "2M7XNZMO5K4ODSZKHQK5",
                TxnType::AppCall,
                "6XHBAFTDDSGTD4AOR67S",
            ),
            Self::block_txn(
                "R2GK3ZGB3B2JPHFY6EZH",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "6HWSA4JGUVDRDDHLQSF3",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "PFKQSP5CLHURDDGEKGBI",
                TxnType::Payment,
                "LE7PTUZJW43AVWQQFX45",
            ),
            Self::block_txn(
                "OWMWSRLGMED46UY7KJDY",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            Self::block_txn(
                "PGT2HOIL67FR26FRWW62",
                TxnType::AssetConfig,
                "ROUNDARTGOAYEGHD4LMD",
            ),
            // Additional transactions to reach 32 total (matching type counts)
            Self::block_txn(
                "TXN14_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER14XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN15_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER15XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN16_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER16XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN17_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER17XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN18_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER18XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN19_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER19XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN20_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER20XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN21_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER21XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN22_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER22XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN23_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER23XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN24_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER24XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN25_APPCALL_XXXXXX",
                TxnType::AppCall,
                "SENDER25XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN26_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER26XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN27_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER27XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN28_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER28XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN29_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER29XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN30_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER30XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN31_ASSETXFR_XXXXX",
                TxnType::AssetTransfer,
                "SENDER31XXXXXXXXXXXXX",
            ),
            Self::block_txn(
                "TXN32_PAYMENT_XXXXXX",
                TxnType::Payment,
                "SENDER32XXXXXXXXXXXXX",
            ),
        ];

        // Build type counts: App Call: 10, Asset Config: 10, Payment: 7, Asset Transfer: 6
        let mut txn_type_counts = HashMap::new();
        txn_type_counts.insert(TxnType::AppCall, 10);
        txn_type_counts.insert(TxnType::AssetConfig, 10);
        txn_type_counts.insert(TxnType::Payment, 7);
        txn_type_counts.insert(TxnType::AssetTransfer, 6);

        BlockDetails {
            info,
            transactions,
            txn_type_counts,
        }
    }

    /// Helper to create a transaction for block details.
    fn block_txn(id_prefix: &str, txn_type: TxnType, from_prefix: &str) -> Transaction {
        Transaction {
            id: id_prefix.to_string(),
            txn_type,
            from: from_prefix.to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "Sat, 17 May 2025 02:37:15".to_string(),
            block: 50_000_000,
            fee: 1000,
            note: String::new(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }
}

// ============================================================================
// JSON Test Data Factories
// ============================================================================

pub struct JsonMother;

impl JsonMother {
    fn base_json(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "sender": "SENDER_ADDRESS",
            "round-time": 1_700_000_000_u64,
            "confirmed-round": 12345_u64,
            "fee": 1000_u64,
        })
    }

    #[must_use]
    pub fn payment() -> serde_json::Value {
        let mut json = Self::base_json("test-payment-id");
        json["payment-transaction"] = serde_json::json!({
            "amount": 5_000_000_u64,
            "receiver": "RECEIVER_ADDRESS"
        });
        json
    }

    #[must_use]
    pub fn payment_with_close() -> serde_json::Value {
        let mut json = Self::base_json("close-txn-id");
        json["payment-transaction"] = serde_json::json!({
            "amount": 5_000_000_u64,
            "receiver": "RECEIVER_ADDRESS",
            "close-remainder-to": "CLOSE_TO_ADDRESS",
            "close-amount": 1_000_000_u64
        });
        json
    }

    #[must_use]
    pub fn asset_transfer() -> serde_json::Value {
        let mut json = Self::base_json("asset-txn-id");
        json["confirmed-round"] = serde_json::json!(12346_u64);
        json["asset-transfer-transaction"] = serde_json::json!({
            "amount": 100_u64,
            "receiver": "RECEIVER_ADDRESS",
            "asset-id": 31_566_704_u64
        });
        json
    }

    #[must_use]
    pub fn asset_transfer_clawback() -> serde_json::Value {
        let mut json = Self::base_json("clawback-txn-id");
        json["sender"] = serde_json::json!("CLAWBACK_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12347_u64);
        json["asset-transfer-transaction"] = serde_json::json!({
            "amount": 50_u64,
            "receiver": "RECEIVER_ADDRESS",
            "asset-id": 31_566_704_u64,
            "sender": "CLAWBACK_TARGET",
            "close-to": "CLOSE_ADDRESS",
            "close-amount": 25_u64
        });
        json
    }

    #[must_use]
    pub fn asset_config_create() -> serde_json::Value {
        let mut json = Self::base_json("asset-create-id");
        json["sender"] = serde_json::json!("CREATOR_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12348_u64);
        json["created-asset-index"] = serde_json::json!(123_456_789_u64);
        json["asset-config-transaction"] = serde_json::json!({
            "params": {
                "total": 1_000_000_u64,
                "decimals": 6_u64,
                "default-frozen": false,
                "name": "Test Token",
                "unit-name": "TEST",
                "url": "https://test.com",
                "metadata-hash": "abc123",
                "manager": "MANAGER_ADDRESS",
                "reserve": "RESERVE_ADDRESS",
                "freeze": "FREEZE_ADDRESS",
                "clawback": "CLAWBACK_ADDRESS"
            }
        });
        json
    }

    #[must_use]
    pub fn asset_config_modify() -> serde_json::Value {
        let mut json = Self::base_json("asset-modify-id");
        json["sender"] = serde_json::json!("MANAGER_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12349_u64);
        json["asset-config-transaction"] = serde_json::json!({
            "asset-id": 123_456_789_u64,
            "params": {
                "manager": "NEW_MANAGER_ADDRESS"
            }
        });
        json
    }

    #[must_use]
    pub fn asset_freeze() -> serde_json::Value {
        let mut json = Self::base_json("freeze-txn-id");
        json["sender"] = serde_json::json!("FREEZE_MANAGER");
        json["confirmed-round"] = serde_json::json!(12350_u64);
        json["asset-freeze-transaction"] = serde_json::json!({
            "address": "TARGET_ADDRESS",
            "asset-id": 31_566_704_u64,
            "new-freeze-status": true
        });
        json
    }

    #[must_use]
    pub fn app_call_create() -> serde_json::Value {
        let mut json = Self::base_json("app-create-id");
        json["sender"] = serde_json::json!("CREATOR_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12351_u64);
        json["created-application-index"] = serde_json::json!(987_654_321_u64);
        json["application-transaction"] = serde_json::json!({
            "application-id": 0_u64,
            "on-completion": "noop",
            "approval-program": "BIAKBQAKAI==",
            "clear-state-program": "BIA=",
            "application-args": ["YXJnMQ==", "YXJnMg=="],
            "accounts": ["ACCOUNT1", "ACCOUNT2"],
            "foreign-apps": [111_u64, 222_u64],
            "foreign-assets": [333_u64, 444_u64],
            "global-state-schema": {
                "num-uint": 10_u64,
                "num-byte-slice": 5_u64
            },
            "local-state-schema": {
                "num-uint": 3_u64,
                "num-byte-slice": 2_u64
            },
            "extra-program-pages": 1_u64
        });
        json
    }

    #[must_use]
    pub fn app_call_with_boxes() -> serde_json::Value {
        let mut json = Self::base_json("app-call-boxes-id");
        json["sender"] = serde_json::json!("CALLER_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12352_u64);
        json["application-transaction"] = serde_json::json!({
            "application-id": 123_456_u64,
            "on-completion": "noop",
            "boxes": [
                {"i": 0_u64, "n": "Ym94MQ=="},
                {"i": 789_u64, "n": "Ym94Mg=="}
            ]
        });
        json
    }

    #[must_use]
    pub fn app_call_on_complete(on_complete: &str) -> serde_json::Value {
        let mut json = Self::base_json(&format!("app-{}-id", on_complete));
        json["confirmed-round"] = serde_json::json!(12353_u64);
        json["application-transaction"] = serde_json::json!({
            "application-id": 123_u64,
            "on-completion": on_complete
        });
        json
    }

    #[must_use]
    pub fn keyreg_online() -> serde_json::Value {
        let mut json = Self::base_json("keyreg-online-id");
        json["sender"] = serde_json::json!("REGISTERING_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12354_u64);
        json["keyreg-transaction"] = serde_json::json!({
            "vote-participation-key": "dm90ZUtleQ==",
            "selection-participation-key": "c2VsS2V5",
            "state-proof-key": "c3BLZXk=",
            "vote-first-valid": 1000_u64,
            "vote-last-valid": 2_000_000_u64,
            "vote-key-dilution": 10_000_u64,
            "non-participation": false
        });
        json
    }

    #[must_use]
    pub fn keyreg_offline() -> serde_json::Value {
        let mut json = Self::base_json("keyreg-offline-id");
        json["sender"] = serde_json::json!("REGISTERING_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12355_u64);
        json["keyreg-transaction"] = serde_json::json!({
            "non-participation": true
        });
        json
    }

    #[must_use]
    pub fn state_proof() -> serde_json::Value {
        let mut json = Self::base_json("state-proof-id");
        json["confirmed-round"] = serde_json::json!(12356_u64);
        json["fee"] = serde_json::json!(0_u64);
        json["state-proof-transaction"] = serde_json::json!({
            "state-proof-type": 0_u64,
            "message": "deadbeef"
        });
        json
    }

    #[must_use]
    pub fn heartbeat() -> serde_json::Value {
        let mut json = Self::base_json("heartbeat-id");
        json["sender"] = serde_json::json!("HEARTBEAT_ADDRESS");
        json["confirmed-round"] = serde_json::json!(12357_u64);
        json["fee"] = serde_json::json!(0_u64);
        json["heartbeat-transaction"] = serde_json::json!({
            "hb-address": "HEARTBEAT_TARGET",
            "hb-key-dilution": 10_000_u64,
            "hb-proof": "cHJvb2Y=",
            "hb-seed": "c2VlZA==",
            "hb-vote-id": "dm90ZUlk"
        });
        json
    }

    #[must_use]
    pub fn empty() -> serde_json::Value {
        serde_json::json!({})
    }
}
