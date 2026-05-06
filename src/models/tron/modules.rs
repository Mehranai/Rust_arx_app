use clickhouse::Row;
use serde::Serialize;

#[derive(Debug, Row, Serialize)]
pub struct TronTransactionRow {
    pub hash: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub contract_index: u32,
    pub owner_addr: String,
    pub to_addr: String,
    pub value: String,
    pub sensivity: u8,
    pub contract_type: String,
    pub classification: Option<String>,
    pub energy_used: u64,
    pub net_used: u64,
    pub fee: String,
}

// models/tron_token_transfer.rs
#[derive(Debug, Row, Serialize)]
pub struct TronTokenTransferRow {
    pub tx_hash: String,
    pub block_number: u64,
    pub log_index: u32,
    pub token_address: String,
    pub from_addr: String,
    pub to_addr: String,
    pub amount: String,
    pub event_signature: String,
}

// models/tron_address_energy.rs
#[derive(Debug, Row, Serialize)]
pub struct TronAddressEnergyRow {
    pub address: String,
    pub block_number: u64,
    pub energy_usage: u64,
    pub energy_fee: u64,
    pub net_usage: u64,
    pub net_fee: u64,
    pub tx_hash: String,
    pub timestamp: u64,
}

// models/tron_contract_call.rs
#[derive(Debug, Row, Serialize)]
pub struct TronContractCallRow {
    pub tx_hash: String,
    pub block_number: u64,
    pub caller_address: String,
    pub contract_address: String,
    pub contract_type: String,
    pub method_signature: String,
    pub call_value: String,
    pub energy_used: u64,
    pub result: String,
    pub timestamp: u64,
}

// models/tron_raw_log.rs
#[derive(Debug, Row, Serialize)]
pub struct TronRawLogRow {
    pub tx_hash: String,
    pub block_number: u64,
    pub log_index: u32,
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub removed: bool,
}

// models/tron_classified_event.rs
#[derive(Debug, Row, Serialize)]
pub struct TronClassifiedEventRow {
    pub event_id: String,
    pub tx_hash: String,
    pub block_number: u64,
    pub event_type: String, // "swap", "bridge_in", "bridge_out", "liquidity_add", "liquidity_remove"
    pub user_address: String,
    pub token_in: Option<String>,
    pub token_out: Option<String>,
    pub amount_in: Option<String>,
    pub amount_out: Option<String>,
    pub protocol: Option<String>,
    pub confidence_score: f32,
    pub timestamp: u64,
}

// transaction_risk.rs
#[derive(Debug, Row, Serialize)]
pub struct TransactionRiskRow {
    pub tx_hash: String,
    pub block_number: u64,
    pub risk_score: u8,
    pub risk_level: String,
    pub is_swap: u8,
    pub is_bridge: u8,
    pub is_contract_call: u8,
    pub unique_tokens: u16,
    pub participants: u16,
}