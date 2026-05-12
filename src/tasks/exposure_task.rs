use anyhow::Result;
use std::sync::Arc;
use crate::services::loader::LoaderTron;
use crate::services::tron::exposure::propagation::propagate_exposure;
use crate::progress::progress_tron::save_address_exposure;

pub async fn run_exposure_scan(
    loader: Arc<LoaderTron>,
    seed: &str,
)
    -> Result<()>
{
    let rows =
        propagate_exposure(

            &loader.clickhouse,

            seed,

            5,
        ).await?;

    for row in rows {
        save_address_exposure(
            loader.clickhouse.clone(),
            row,
        ).await?;
    }

    Ok(())
}