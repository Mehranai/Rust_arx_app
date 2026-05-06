use crate::utils::tron_classifier::types::ContractType;

pub fn compute_risk_score(
    classification: &ContractType,
    is_swap: bool,
    is_bridge: bool,
    unique_tokens: u16,
    participants: u16,
) -> (u8, String) {

    let mut score: u8 = 0;

    // 🔥 base signals
    if is_bridge {
        score += 40; // bridges are high risk
    }

    if is_swap {
        score += 10;
    }

    // 🔥 contract intelligence
    match classification {
        ContractType::Dex => score += 10,
        ContractType::Bridge => score += 30,
        ContractType::Lending => score += 20,
        ContractType::Scam => score += 80,
        _ => {}
    }

    // 🔥 complexity signals
    if unique_tokens > 2 {
        score += 15;
    }

    if participants > 3 {
        score += 15;
    }

    // clamp
    if score > 100 {
        score = 100;
    }

    let level = match score {
        0..=30 => "LOW",
        31..=70 => "MEDIUM",
        _ => "HIGH",
    }.to_string();

    (score, level)
}