use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use serde_json::Value;

use crate::models::tron::modules::TronTokenTransferRow;
use crate::models::tron::modules::TransactionRiskRow;

use crate::progress::progress::{
    save_sync_state,
    save_wallet,
};

use crate::progress::progress_tron::{
    save_contract_metadata,
    save_transaction_features,
    save_transaction_risk,
    save_token_transfer,
    save_tx,
    ContractMetadataRow,
    TransactionFeatureRow,
};

use crate::services::loader::LoaderTron;

use crate::utils::tron_address::normalize_tron_address;

use crate::services::tron::tron_classification::{
    detect_bridges,
    detect_swaps,
    SimpleTransfer,
};

use crate::services::tron::tron_classifier::classifier::classify;
use crate::services::tron::tron_classifier::types::{
    ClassificationInput,
    ContractType,
};

use crate::services::tron::tron_metadata_worker;
use crate::services::tron::tron_risk_engine::compute_risk_score;

const ZERO_ADDRESS: &str =
    "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb";

const ERC20_TRANSFER_TOPIC: &str =
    "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

fn calc_sensivity_tron(value_sun: u64) -> u8 {
    let trx = value_sun as f64 / 1_000_000.0;

    if trx > 50_000.0 {
        2
    } else if trx > 5_000.0 {
        1
    } else {
        0
    }
}

fn extract_contract_type(tx: &Value) -> String {
    tx["raw_data"]["contract"][0]["type"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string()
}

fn extract_transfer_contract(
    tx: &Value,
) -> Option<(String, String, u64)> {
    let contract = &tx["raw_data"]["contract"][0];

    if contract["type"] != "TransferContract" {
        return None;
    }

    let value = contract["parameter"]["value"]["amount"]
        .as_u64()?;

    let owner = contract["parameter"]["value"]["owner_address"]
        .as_str()?;

    let to = contract["parameter"]["value"]["to_address"]
        .as_str()?;

    let from = normalize_tron_address(owner)?;
    let to = normalize_tron_address(to)?;

    Some((from, to, value))
}

fn extract_trc20_transfers(
    receipt: &Value,
) -> Vec<(u32, String, String, String, u128)> {
    let mut transfers = Vec::new();

    let empty_logs = Vec::new();

    let logs = receipt["log"]
        .as_array()
        .unwrap_or(&empty_logs);

    for (i, log) in logs.iter().enumerate() {

        let empty_topics = Vec::new();

        let topics = log["topics"]
            .as_array()
            .unwrap_or(&empty_topics);

        if topics.len() < 3 {
            continue;
        }

        let topic0 = topics[0]
            .as_str()
            .unwrap_or("")
            .to_lowercase();

        if topic0 != ERC20_TRANSFER_TOPIC {
            continue;
        }

        let token = log["address"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let from = normalize_tron_address(
            topics[1].as_str().unwrap_or("")
        );

        let to = normalize_tron_address(
            topics[2].as_str().unwrap_or("")
        );

        let amount_hex = log["data"]
            .as_str()
            .unwrap_or("0x0");

        let amount = u128::from_str_radix(
            amount_hex.trim_start_matches("0x"),
            16,
        )
            .unwrap_or(0);

        if let (Some(from), Some(to)) = (from, to) {
            transfers.push((
                i as u32,
                token,
                from,
                to,
                amount,
            ));
        }
    }

    transfers
}

async fn save_wallet_tron(
    loader: Arc<LoaderTron>,
    addr: String,
) -> Result<()> {

    if addr.is_empty() {
        return Ok(());
    }

    let account = {
        let _permit =
            loader.rpc_limiter.acquire().await?;

        loader
            .tron_client
            .get_account(&addr)
            .await?
    };

    let balance = account["balance"]
        .as_u64()
        .unwrap_or(0);

    let nonce = account["latest_opration_time"]
        .as_u64()
        .unwrap_or(0);

    let account_type = account["type"]
        .as_str()
        .unwrap_or("");

    let wallet_type = match account_type {
        "Contract" => "smart_contract",
        _ => "wallet",
    }
        .to_string();

    save_wallet(
        loader.clickhouse.clone(),
        &addr,
        balance.to_string(),
        nonce,
        wallet_type,
    )
        .await?;

    Ok(())
}

async fn process_tx(
    loader: Arc<LoaderTron>,
    tx: Value,
    block_number: u64,
) -> Result<()> {

    let txid = tx["txID"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing txID"))?
        .to_string();

    let contract_type =
        extract_contract_type(&tx);

    let mut from = String::new();
    let mut to = String::new();
    let mut value = 0u64;

    if let Some((f, t, v)) =
        extract_transfer_contract(&tx)
    {
        from = f;
        to = t;
        value = v;
    }

    save_tx(
        loader.clickhouse.clone(),
        txid.clone(),
        block_number,
        from.clone(),
        to.clone(),
        value.to_string(),
        contract_type.clone(),
        calc_sensivity_tron(value),
    ).await?;

    let receipt = {
        let _permit =
            loader.rpc_limiter.acquire().await?;

        loader
            .tron_client
            .get_tx_receipt(&txid)
            .await?
    };

    let transfers =
        extract_trc20_transfers(&receipt);

    let mut simple_transfers =
        Vec::<SimpleTransfer>::new();

    let mut discovered_tokens =
        HashSet::<String>::new();

    for (
        log_index,
        token,
        from_addr,
        to_addr,
        amount,
    ) in transfers
    {
        save_token_transfer(
            loader.clickhouse.clone(),
            TronTokenTransferRow {
                tx_hash: txid.clone(),
                block_number,
                log_index,
                token_address: token.clone(),
                from_addr: from_addr.clone(),
                to_addr: to_addr.clone(),
                amount: amount.to_string(),
                event_signature: ERC20_TRANSFER_TOPIC.to_string(),
            },
        )
            .await?;

        discovered_tokens.insert(token.clone());

        if from_addr != ZERO_ADDRESS
            && to_addr != ZERO_ADDRESS
        {
            simple_transfers.push(
                SimpleTransfer {
                    token,
                    from: from_addr,
                    to: to_addr,
                    amount,
                },
            );
        }
    }

    // token metadata worker
    if !discovered_tokens.is_empty() {

        let tokens: Vec<String> =
            discovered_tokens
                .into_iter()
                .collect();

        tron_metadata_worker::process_new_tokens(
            loader.clone(),
            tokens,
        )
            .await?;
    }

    // contract classifier
    let contract_address = tx["raw_data"]["contract"][0]
        ["parameter"]["value"]["contract_address"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let method_data = tx["raw_data"]["contract"][0]
        ["parameter"]["value"]["data"]
        .as_str()
        .map(|s| s.to_string());

    let classification = classify(
        &ClassificationInput {
            contract_address:
            contract_address.clone(),
            method_data,
        },
        &simple_transfers,
    );

    let is_contract_call = match classification {
        ContractType::Dex
        | ContractType::Bridge
        | ContractType::Lending => 1,

        _ => {
            if contract_type
                == "TriggerSmartContract"
            {
                1
            } else {
                0
            }
        }
    };

    // save contract metadata
    if !contract_address.is_empty() {

        let row = ContractMetadataRow {
            contract_address:
            contract_address.clone(),

            contract_type:
            classification.to_string(),

            creator_address: from.clone(),

            created_at_block: block_number,
        };

        save_contract_metadata(
            loader.clickhouse.clone(),
            row,
        )
            .await?;
    }

    // AML features
    if !simple_transfers.is_empty() {

        let swaps =
            detect_swaps(
                &simple_transfers
            );

        let bridges =
            detect_bridges(
                &simple_transfers
            );

        let unique_tokens =
            simple_transfers
                .iter()
                .map(|t| t.token.clone())
                .collect::<HashSet<_>>()
                .len() as u16;

        let participants =
            simple_transfers
                .iter()
                .flat_map(|t| {
                    vec![
                        t.from.clone(),
                        t.to.clone(),
                    ]
                })
                .collect::<HashSet<_>>()
                .len() as u16;

        let feature =
            TransactionFeatureRow {
                tx_hash: txid.clone(),
                block_number,

                is_swap:
                (!swaps.is_empty()) as u8,

                is_bridge:
                (!bridges.is_empty()) as u8,

                is_contract_call,

                unique_tokens,
                participants,
            };

        save_transaction_features(
            loader.clickhouse.clone(),
            feature,
        )
            .await?;

        // risk engine
        let (
            risk_score,
            risk_level,
        ) = compute_risk_score(
            &classification,
            !swaps.is_empty(),
            !bridges.is_empty(),
            unique_tokens,
            participants,
        );

        let risk_row =
            TransactionRiskRow {
                tx_hash: txid.clone(),
                block_number,

                risk_score,
                risk_level,

                is_swap:
                (!swaps.is_empty()) as u8,

                is_bridge:
                (!bridges.is_empty()) as u8,

                is_contract_call,

                unique_tokens,
                participants,
            };

        save_transaction_risk(
            loader.clickhouse.clone(),
            risk_row,
        )
            .await?;
    }

    // save all wallets
    let mut wallets =
        HashSet::<String>::new();

    if !from.is_empty() {
        wallets.insert(from);
    }

    if !to.is_empty() {
        wallets.insert(to);
    }

    for t in &simple_transfers {
        wallets.insert(t.from.clone());
        wallets.insert(t.to.clone());
    }

    for addr in wallets {
        save_wallet_tron(
            loader.clone(),
            addr,
        )
            .await?;
    }

    Ok(())
}

pub async fn fetch_tron(
    loader: Arc<LoaderTron>,
    start_block: u64,
    total_txs: u64,
) -> Result<()> {

    let latest_block =
        loader
            .tron_client
            .get_block_number()
            .await?;

    println!(
        "TRON Latest Block: {}",
        latest_block
    );

    let mut tx_count = 0u64;

    let mut current_block =
        start_block;

    let mut last_synced_block =
        start_block;

    while current_block <= latest_block {

        if tx_count >= total_txs {
            break;
        }

        let block = {
            let _permit =
                loader
                    .rpc_limiter
                    .acquire()
                    .await?;

            loader
                .tron_client
                .get_block(current_block)
                .await?
        };

        let empty_txs = Vec::new();

        let txs = block["transactions"]
            .as_array()
            .unwrap_or(&empty_txs);

        if txs.is_empty() {

            last_synced_block =
                current_block;

            save_sync_state(
                loader.clickhouse.clone(),
                "tron",
                last_synced_block,
            )
                .await?;

            current_block += 1;

            continue;
        }

        let mut tasks =
            FuturesUnordered::new();

        let mut fully_processed = true;

        for tx in txs {

            if tx_count >= total_txs {

                fully_processed = false;

                break;
            }

            let loader_clone =
                loader.clone();

            let tx_clone = tx.clone();

            tasks.push(
                tokio::spawn(async move {
                    process_tx(
                        loader_clone,
                        tx_clone,
                        current_block,
                    )
                        .await
                }),
            );

            tx_count += 1;

            println!(
                "[TRON] queued tx #{}",
                tx_count
            );
        }

        while let Some(res) =
            tasks.next().await
        {
            res??;
        }

        if fully_processed {

            last_synced_block =
                current_block;

            save_sync_state(
                loader.clickhouse.clone(),
                "tron",
                last_synced_block,
            )
                .await?;

            println!(
                "TRON synced block {} | total tx {}",
                last_synced_block,
                tx_count
            );

        } else {

            println!(
                "TRON stopped mid-block {} | total tx {}",
                current_block,
                tx_count
            );

            break;
        }

        current_block += 1;
    }

    save_sync_state(
        loader.clickhouse.clone(),
        "tron",
        last_synced_block,
    )
        .await?;

    Ok(())
}