use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::stream::{self, StreamExt};
use serde_json::Value;

use crate::models::tron::modules::TronTokenTransferRow;
use crate::models::tron::modules::TransactionRiskRow;
use crate::models::tron::modules::TransactionRow;

use crate::progress::progress::{
    save_sync_state,
    save_wallet,
};

use crate::progress::progress_tron::{
    ContractMetadataRow,
};

use crate::models::tron::modules::{
    TransactionFeatureRow,
};

use crate::services::loader::LoaderTron;
use crate::utils::tron_address::normalize_tron_address;

// aml section
use crate::services::tron::aml::types::SimpleTransfer;
use crate::services::tron::aml::swap_detector::detect_swaps;
use crate::services::tron::aml::bridge_detector::detect_bridges;

use crate::services::tron::tron_classifier::classifier::classify;
use crate::services::tron::tron_classifier::types::{
    ClassificationInput,
    ContractCategory,
};

use crate::services::tron::tron_metadata_worker;
use crate::services::tron::risk_engine::compute_risk_score;

use crate::services::tron::relationship_builder::build_relationships;
use crate::services::tron::aml::mint_burn_detector::detect_mints_and_burns;

// flow detection
use crate::services::tron::exchange::detector::{
    detect_exchange,
    detect_exchange_attributions,
};
use crate::services::tron::exchange::flow_builder::build_exchange_flows;
use crate::models::tron::exchange::ExchangeAddressRow;

// intelligence system
use crate::services::tron::address_intelligence::{
    build_address_profiles,
};
use crate::models::tron::address_profile::AddressProfileRow;
use crate::progress::address_profile::{
    save_address_profiles,
};
use crate::models::tron::counterparty::CounterpartyRow;
use crate::progress::counterparty::save_counterparties;
use crate::services::tron::counterparty::build_counterparty_relations;

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

    let mut simple_transfers =
        Vec::<SimpleTransfer>::new();

    if !from.is_empty()
        && !to.is_empty()
        && value > 0
    {
        simple_transfers.push(
            SimpleTransfer {

                token: "TRX".to_string(),

                from: from.clone(),
                to: to.clone(),

                amount: value as u128,
            }
        );
    }

    let receipt = {
        let _permit =
            loader.rpc_limiter.acquire().await?;

        loader
            .tron_client
            .get_tx_receipt(&txid)
            .await?
    };

    let timestamp = tx["raw_data"]["timestamp"]
        .as_u64()
        .unwrap_or(0);

    let receipt_result = receipt["receipt"]["result"]
        .as_str()
        .unwrap_or("");

    let status = if receipt_result == "SUCCESS" {
        1
    } else {
        0
    };

    let fee = receipt["fee"]
        .as_u64()
        .unwrap_or(0) as u128;

    let energy_fee = receipt["energy_fee"]
        .as_u64()
        .unwrap_or(0) as u128;

    let net_fee = receipt["net_fee"]
        .as_u64()
        .unwrap_or(0) as u128;

    let energy_usage = receipt["receipt"]["energy_usage"]
        .as_u64()
        .unwrap_or(0);

    let energy_usage_total =
        receipt["receipt"]["energy_usage_total"]
            .as_u64()
            .unwrap_or(0);

    let net_usage = receipt["receipt"]["net_usage"]
        .as_u64()
        .unwrap_or(0);

    // contract classifier
    let contract_address = tx["raw_data"]["contract"][0]
        ["parameter"]["value"]["contract_address"]
        .as_str()
        .unwrap_or("")
        .to_string();

    loader.transaction_batcher
        .push(
            TransactionRow {
                tx_hash: txid.clone(),
                block_number,
                timestamp,

                from_address: from.clone(),
                to_address: to.clone(),

                contract_address: contract_address.clone(),

                contract_type: contract_type.clone(),

                amount: value as u128,
                fee,
                energy_fee,
                net_fee,
                energy_usage,
                energy_usage_total,
                net_usage,
                status,
                memo: String::new(),

                raw_data: tx.to_string(),
            }
        )
        .await?;

    let transfers =
        extract_trc20_transfers(&receipt);

    let mut discovered_tokens =
        HashSet::<String>::new();

    for (
        log_index,
        token,
        from_addr,
        to_addr,
        amount,
    ) in transfers {
        loader
            .token_transfer_batcher
            .push(
                TronTokenTransferRow {
                    tx_hash: txid.clone(),
                    block_number,
                    timestamp,

                    log_index,

                    token_address: token.clone(),

                    token_symbol: String::new(),

                    decimals: 0,

                    from_address: from_addr.clone(),

                    to_address: to_addr.clone(),

                    amount,

                    amount_decimal: 0.0,

                    is_mint:
                    (from_addr == ZERO_ADDRESS) as u8,

                    is_burn:
                    (to_addr == ZERO_ADDRESS) as u8,

                    event_signature:
                    ERC20_TRANSFER_TOPIC.to_string(),
                }
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

    let is_contract_call = match classification.category {
        ContractCategory::Dex
        | ContractCategory::Bridge
        | ContractCategory::Lending => 1,

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
            classification.category.to_string(),

            creator_address: from.clone(),

            created_at_block: block_number,
        };

        loader
            .contract_metadata_batcher
            .push(row)
            .await?;
    }

    // AML features
    if !simple_transfers.is_empty() {

        let swaps =
            detect_swaps(
                &simple_transfers
            );
        let mint_burns =
            detect_mints_and_burns(
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

                timestamp,

                is_swap:
                (!swaps.is_empty()) as u8,

                is_bridge:
                (!bridges.is_empty()) as u8,

                is_mint:
                (!mint_burns.is_empty()) as u8,

                is_burn:
                (!mint_burns.is_empty()) as u8,

                is_liquidity_add: 0,

                is_liquidity_remove: 0,

                is_contract_call,

                unique_tokens,

                participants,

                hop_count:
                simple_transfers.len() as u16,

                fan_in:
                participants,

                fan_out:
                participants,
            };


        let mut aml_events = Vec::new();
        aml_events.extend(swaps.clone());
        aml_events.extend(bridges.clone());
        aml_events.extend(mint_burns.clone());

        loader
            .transaction_feature_batcher
            .push(feature)
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
                timestamp,

                risk_score,
                risk_level,

                is_swap: (!swaps.is_empty()) as u8,
                is_bridge: (!bridges.is_empty()) as u8,
                is_contract_call,

                unique_tokens,
                participants,

                risk_reasons: vec![],
                exposure_depth: 0,
                touches_sanctioned: 0,
                touches_mixer: 0,
                touches_exchange: 0,
            };
        loader
            .transaction_risk_batcher
            .push(risk_row)
            .await?;

        let relationships =
            build_relationships(
                &txid,
                block_number,
                tx["raw_data"]["timestamp"]
                    .as_u64()
                    .unwrap_or(0),

                &simple_transfers,
                &aml_events,
                &classification.protocol,
                risk_score,
            );

        for row in relationships {
            loader
                .relationship_batcher
                .push(row)
                .await?;
        }

        // historical address intelligence
        let profiles =
            build_address_profiles(
                &simple_transfers
            );
        let mut profile_rows =
            Vec::<AddressProfileRow>::new();

        for (_, profile) in profiles {

            profile_rows.push(
                AddressProfileRow {
                    address: profile.address,
                    total_in_tx: profile.total_in_tx,
                    total_out_tx: profile.total_out_tx,
                    unique_senders: profile.unique_senders,
                    unique_receivers: profile.unique_receivers,
                    total_volume_in: profile.total_volume_in,
                    total_volume_out: profile.total_volume_out,
                    interacted_tokens: profile.interacted_tokens.len() as u32,
                    probable_exchange: profile.probable_exchange as u8,
                    probable_deposit_wallet: profile.probable_deposit_wallet as u8,
                    probable_sweeper: profile.probable_sweeper as u8,
                    risk_score: profile.risk_score,
                }
            );
        }
        save_address_profiles(
            loader.clickhouse.clone(),
            profile_rows,
        ).await?;

        let counterparty_rows =
            build_counterparty_relations(
                &simple_transfers,
                block_number,
            )
                .into_iter()
                .map(|relation| {
                    CounterpartyRow {
                        address: relation.address,
                        counterparty: relation.counterparty,
                        direction: relation.direction,
                        token_address: relation.token_address,
                        total_txs: relation.total_txs,
                        total_volume: relation.total_volume,
                        first_seen: relation.first_seen,
                        last_seen: relation.last_seen,
                    }
                })
                .collect::<Vec<_>>();

        save_counterparties(
            loader.clickhouse.clone(),
            counterparty_rows,
        )
            .await?;

        let exchange_flows =
            build_exchange_flows(
                &txid,
                block_number,
                &simple_transfers,
            );

        for flow in exchange_flows {
            loader
                .exchange_flow_batcher
                .push(flow)
                .await?;
        }

        let exchange_detections =
            detect_exchange_attributions(
                loader.clickhouse.clone(),
                block_number,
                &simple_transfers,
            )
                .await?;

        for detection in exchange_detections {
            save_exchange_address(
                loader.clickhouse.clone(),
                detection.address,
            )
                .await?;

            if let Some(deposit) = detection.deposit {
                save_exchange_deposit_address(
                    loader.clickhouse.clone(),
                    deposit,
                )
                    .await?;
            }

            save_exchange_cluster(
                loader.clickhouse.clone(),
                detection.cluster,
            )
                .await?;
        }
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
        if let Some(exchange) =
            detect_exchange(&addr)
        {
            save_exchange_address(
                loader.clickhouse.clone(),
                ExchangeAddressRow {
                    address:
                    addr.clone(),

                    entity_id:
                    format!("exchange:{}", exchange.exchange_name.to_lowercase()),

                    exchange_name:
                    exchange.exchange_name,

                    address_role:
                    exchange.role,

                    confidence:
                    exchange.confidence,

                    detection_source:
                    exchange.detection_source,

                    first_seen_block:
                    block_number,

                    last_seen_block:
                    block_number,
                }
            ).await?;
        }

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

        let mut fully_processed = true;

        let tx_vec = txs
                .iter()
                .cloned()
                .collect::<Vec<_>>();

        let remaining = total_txs.saturating_sub(tx_count);

        let tx_vec = tx_vec
                .into_iter()
                .take(remaining as usize)
                .collect::<Vec<_>>();

        if tx_vec.len() < txs.len()
        {
            fully_processed = false;
        }

        tx_count += tx_vec.len() as u64;

        stream::iter(tx_vec)
            .map(|tx| {

                let loader_clone =
                    loader.clone();

                async move {
                    process_tx(
                        loader_clone,
                        tx,
                        current_block,
                    ).await
                }
            })

            .buffer_unordered(
                loader
                    .config
                    .tx_worker_concurrency
            )
            .for_each(|res| async {
                if let Err(err) = res {
                    eprintln!(
                        "[TRON TX ERROR] {:?}",
                        err
                    );
                }
            }).await;

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
