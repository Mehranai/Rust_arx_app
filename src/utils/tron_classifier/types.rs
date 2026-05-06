use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractType {
    Dex,
    Bridge,
    Lending,
    Token,
    Nft,
    Scam,
    Unknown,
}

impl ToString for ContractType {
    fn to_string(&self) -> String {
        match self {
            ContractType::Dex => "DEX",
            ContractType::Bridge => "BRIDGE",
            ContractType::Lending => "LENDING",
            ContractType::Token => "TRC20",
            ContractType::Nft => "NFT",
            ContractType::Scam => "SCAM",
            ContractType::Unknown => "UNKNOWN",
        }.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct ClassificationInput {
    pub contract_address: String,
    pub method_data: Option<String>,
}