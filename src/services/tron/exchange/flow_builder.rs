use std::collections::HashMap;
use crate::services::tron::aml::types::SimpleTransfer;

pub fn build_exchange_flows(
    transfers: &[SimpleTransfer],
)
    -> HashMap<String, Vec<SimpleTransfer>>
{

    let mut flows =
        HashMap::<
            String,
            Vec<SimpleTransfer>
        >::new();

    for transfer in transfers {

        flows
            .entry(
                transfer.from.clone()
            )
            .or_default()
            .push(
                transfer.clone()
            );

        flows
            .entry(
                transfer.to.clone()
            )
            .or_default()
            .push(
                transfer.clone()
            );
    }

    flows
}