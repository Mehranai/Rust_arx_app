use std::collections::HashMap;
use super::types::ContractType;

pub fn known_contracts() -> HashMap<&'static str, ContractType> {
    let mut map = HashMap::new();

    // 🔥 You MUST grow this over time
    map.insert("TV7o...replace", ContractType::Dex);     // SunSwap
    map.insert("TJ...replace", ContractType::Lending);   // JustLend

    map
}