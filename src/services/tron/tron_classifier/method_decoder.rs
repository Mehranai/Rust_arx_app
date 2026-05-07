use super::types::ContractType;

pub fn detect_method(method_data: &str) -> Option<ContractType> {
    if method_data.len() < 8 {
        return None;
    }

    let method_id = &method_data[0..8];

    match method_id {
        // swapExactTokensForTokens
        "38ed1739" => Some(ContractType::Dex),

        // swapExactETHForTokens
        "7ff36ab5" => Some(ContractType::Dex),

        // addLiquidity
        "e8e33700" => Some(ContractType::Dex),

        // lending borrow
        "c5ebeaec" => Some(ContractType::Lending),

        _ => None,
    }
}