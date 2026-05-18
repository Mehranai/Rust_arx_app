use crate::models::tron::exchange::ExchangeFlowRow;
use crate::services::tron::aml::types::SimpleTransfer;
use crate::services::tron::exchange::detector::detect_exchange;

pub fn build_exchange_flows(
    tx_hash: &str,
    block_number: u64,
    transfers: &[SimpleTransfer],
) -> Vec<ExchangeFlowRow> {
    let mut flows = Vec::new();

    for transfer in transfers {
        if let Some(exchange) = detect_exchange(&transfer.to) {
            flows.push(ExchangeFlowRow {
                tx_hash: tx_hash.to_string(),
                block_number,
                from_address: transfer.from.clone(),
                to_address: transfer.to.clone(),
                exchange_name: exchange.exchange_name,
                flow_type: "deposit".to_string(),
                token_address: transfer.token.clone(),
                amount: transfer.amount.to_string(),
                confidence: exchange.confidence,
            });
        }

        if let Some(exchange) = detect_exchange(&transfer.from) {
            flows.push(ExchangeFlowRow {
                tx_hash: tx_hash.to_string(),
                block_number,
                from_address: transfer.from.clone(),
                to_address: transfer.to.clone(),
                exchange_name: exchange.exchange_name,
                flow_type: "withdrawal".to_string(),
                token_address: transfer.token.clone(),
                amount: transfer.amount.to_string(),
                confidence: exchange.confidence,
            });
        }
    }

    flows
}
