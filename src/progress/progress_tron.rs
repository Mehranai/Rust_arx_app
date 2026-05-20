use clickhouse::Client;
use std::sync::Arc;

use crate::models::tron::exposure::{
    AddressExposureRow,
    ExposureSeedRow,
};


// --------------------------------------------------
// CONTRACT METADATA
// --------------------------------------------------

#[derive(Debug, Clone, clickhouse::Row, serde::Serialize, )]
pub struct ContractMetadataRow {

    pub contract_address: String,

    pub contract_type: String,

    pub creator_address: String,

    #[serde(rename = "created_block")]
    pub created_at_block: u64,
}

// --------------------------------------------------
// LOW FREQUENCY UTILITIES
// --------------------------------------------------

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