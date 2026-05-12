use std::sync::Arc;
use clickhouse::Client;

use crate::models::tron::modules::{TransactionRow, TronTokenTransferRow};
use crate::models::tron::modules::TransactionRiskRow;

use crate::models::tron::relationship::AddressRelationshipRow;

use crate::models::tron::exchange::{
    ExchangeAddressRow,
    ExchangeFlowRow,
};

use crate::models::tron::exposure::{
    ExposureSeedRow,
    AddressExposureRow,
};

pub async fn save_tx(
    clickhouse: Arc<Client>,
    hash: String,
    block_number: u64,
    from: String,
    to: String,
    value: String,
    contract_type: String,
    sensivity: u8,
) -> anyhow::Result<()> {

    let tx_row = TransactionRow {
        hash,
        block_number,
        from_addr: from,
        to_addr: to,
        value,
        contract_type,
        sensivity,
    };

    let mut insert =
        clickhouse.insert::<TransactionRow>("transactions").await?;

    insert.write(&tx_row).await?;
    insert.end().await?;

    Ok(())
}

//
// --------------------------------------------------
// AML FEATURES
// --------------------------------------------------
//

#[derive(Debug, clickhouse::Row, serde::Serialize)]
pub struct TransactionFeatureRow {
    pub tx_hash: String,
    pub block_number: u64,
    pub is_swap: u8,
    pub is_bridge: u8,
    pub is_contract_call: u8,
    pub unique_tokens: u16,
    pub participants: u16,
}

pub async fn save_transaction_features(
    clickhouse: Arc<Client>,
    row: TransactionFeatureRow,
) -> anyhow::Result<()> {

    let mut insert = clickhouse
        .insert::<TransactionFeatureRow>("transaction_features")
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

//
// --------------------------------------------------
// CONTRACT METADATA
// --------------------------------------------------
//

#[derive(Debug, clickhouse::Row, serde::Serialize)]
pub struct ContractMetadataRow {
    pub contract_address: String,
    pub contract_type: String,
    pub creator_address: String,
    pub created_at_block: u64,
}

pub async fn save_contract_metadata(
    clickhouse: Arc<Client>,
    row: ContractMetadataRow,
) -> anyhow::Result<()> {

    let existing: u64 = clickhouse
        .query(
            "
        SELECT count()
        FROM contract_metadata
        WHERE contract_address = ?
        "
        )
        .bind(&row.contract_address)
        .fetch_one::<u64>()
        .await?;

    if existing > 0 {
        return Ok(());
    }

    let mut insert = clickhouse
        .insert::<ContractMetadataRow>("contract_metadata")
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

// --------------------------------------------------
// TRANSACTION RISK
// --------------------------------------------------

pub async fn save_transaction_risk(
    clickhouse: Arc<Client>,
    row: TransactionRiskRow,
) -> anyhow::Result<()> {

    let mut insert = clickhouse
        .insert::<TransactionRiskRow>("transaction_risk")
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

pub async fn save_token_transfer(
    clickhouse: Arc<Client>,
    row: TronTokenTransferRow,
) -> anyhow::Result<()> {

    let mut insert =
        clickhouse.insert::<TronTokenTransferRow>("token_transfers").await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

pub async fn save_relationships(
    clickhouse: Arc<Client>,
    rows: Vec<AddressRelationshipRow>,
)
    -> anyhow::Result<()>
{
    if rows.is_empty() {
        return Ok(());
    }

    let mut insert = clickhouse
        .insert::<AddressRelationshipRow>(
            "address_relationships"
        )
        .await?;

    for row in rows {
        insert.write(&row).await?;
    }

    insert.end().await?;

    Ok(())
}

pub async fn save_exchange_address(
    clickhouse: Arc<Client>,
    row: ExchangeAddressRow,
)
    -> anyhow::Result<()>
{

    let mut insert = clickhouse
        .insert::<ExchangeAddressRow>(
            "exchange_addresses"
        )
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

pub async fn save_exchange_flow(
    clickhouse: Arc<Client>,
    row: ExchangeFlowRow,
)
    -> anyhow::Result<()>
{

    let mut insert = clickhouse
        .insert::<ExchangeFlowRow>(
            "exchange_flows"
        )
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

pub async fn save_exposure_seed(
    clickhouse: Arc<Client>,
    row: ExposureSeedRow,
)
    -> anyhow::Result<()>
{
    let mut insert = clickhouse
        .insert::<ExposureSeedRow>(
            "exposure_seeds"
        )
        .await?;
    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}

pub async fn save_address_exposure(
    clickhouse: Arc<Client>,
    row: AddressExposureRow,
)
    -> anyhow::Result<()>
{
    let mut insert = clickhouse
        .insert::<AddressExposureRow>(
            "address_exposure"
        )
        .await?;

    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}