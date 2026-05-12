use super::seeds::exchange_seeds;

use super::types::{
    ExchangeAttribution,
    ExchangeWalletRole,
};

pub fn detect_exchange(
    address: &str,
)
    -> Option<ExchangeAttribution>
{

    //
    // seed matching
    //

    if let Some(attr) =
        exchange_seeds().get(address)
    {
        return Some(attr.clone());
    }

    None
}

pub fn build_deposit_attribution(

    exchange_name: &str,
)
    -> ExchangeAttribution
{

    ExchangeAttribution {

        exchange_name:
        exchange_name.to_string(),

        role:
        ExchangeWalletRole::Deposit
            .to_string(),

        confidence: 0.65,

        detection_source:
        "deposit_heuristic"
            .to_string(),

        cluster_id: None,
    }
}