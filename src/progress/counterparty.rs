use std::sync::Arc;
use clickhouse::Client;
use crate::models::tron::counterparty::CounterpartyRow;

pub async fn save_counterparties(
    clickhouse:Arc<Client>,
    rows:Vec<CounterpartyRow>,
)->anyhow::Result<()>
{
    if rows.is_empty() {
        return Ok(());
    }

    let mut insert=
        clickhouse
            .insert::<CounterpartyRow>(
                "tron_aml.address_counterparties"
            )
            .await?;

    for row in rows {
        insert.write(&row).await?;
    }

    insert.end().await?;

    Ok(())
}