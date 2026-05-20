use crate::models::tron::modules::TransactionRiskRow;

use super::generic::GenericBatcher;

pub type TransactionRiskBatcher =
GenericBatcher<TransactionRiskRow>;