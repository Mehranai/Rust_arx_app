use super::types::{ContractType, ClassificationInput};
use super::registry::known_contracts;
use super::method_decoder::detect_method;
use super::flow_analyzer::analyze_flows;

use crate::utils::tron_classification::SimpleTransfer;

pub fn classify(
    input: &ClassificationInput,
    transfers: &[SimpleTransfer],
) -> ContractType {

    // 1. Known contracts
    if let Some(t) = known_contracts().get(input.contract_address.as_str()) {
        return t.clone();
    }

    // 2. Method decoding
    if let Some(ref data) = input.method_data {
        if let Some(t) = detect_method(data) {
            return t;
        }
    }

    // 3. Flow analysis fallback
    if let Some(t) = analyze_flows(transfers) {
        return t;
    }

    ContractType::Unknown
}