use super::types::ContractType;
use crate::services::tron::tron_classification::SimpleTransfer;

pub fn analyze_flows(transfers: &[SimpleTransfer]) -> Option<ContractType> {

    let mut sent_tokens = std::collections::HashSet::new();
    let mut received_tokens = std::collections::HashSet::new();

    for t in transfers {
        sent_tokens.insert(t.token.clone());
        received_tokens.insert(t.token.clone());
    }

    // swap pattern
    if sent_tokens.len() > 0 && received_tokens.len() > 1 {
        return Some(ContractType::Dex);
    }

    None
}