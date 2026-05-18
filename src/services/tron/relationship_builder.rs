use crate::models::tron::relationship::AddressRelationshipRow;

use crate::services::tron::aml::types::{
    AmlEvent,
    SimpleTransfer,
};

use crate::services::tron::relationship_types::RelationshipType;

pub fn build_relationships(

    tx_hash: &str,
    block_number: u64,
    timestamp: u64,
    transfers: &[SimpleTransfer],
    events: &[AmlEvent],
    protocol: &str,
    risk_score: u8,
)
    -> Vec<AddressRelationshipRow>
{

    let mut rows = Vec::new();
    //
    // raw value-transfer edges
    //
    for transfer in transfers {
        let transfer_type = if transfer.token == "TRX" {
            RelationshipType::NativeTransfer
        } else {
            RelationshipType::Trc20Transfer
        };

        rows.push(
            AddressRelationshipRow {
                from_address: transfer.from.clone(),
                to_address: transfer.to.clone(),
                token_address: transfer.token.clone(),
                tx_hash: tx_hash.to_string(),
                block_number,
                timestamp,
                amount: transfer.amount,
                transfer_type: transfer_type.to_string(),
                protocol: protocol.to_string(),
                risk_score,
            }
        );
    }
    //
    // semantic AML events
    //
    for event in events {
        match event {
            //
            // SWAPS
            //
            AmlEvent::Swap {
                user,
                token_in,
                token_out,
            } => {

                rows.push(
                    AddressRelationshipRow {
                        from_address: user.clone(),
                        to_address: protocol.to_string(),
                        token_address: format!("{}:{}", token_in, token_out),
                        tx_hash: tx_hash.to_string(),
                        block_number,
                        timestamp,
                        amount: 0,
                        transfer_type: RelationshipType::Swap.to_string(),
                        protocol: protocol.to_string(),
                        risk_score,
                    }
                );
            }
            //
            // BRIDGES
            //
            AmlEvent::BridgeIn {
                user,
                token,
            } => {
                rows.push(
                    AddressRelationshipRow {
                        from_address: "bridge".to_string(),
                        to_address: user.clone(),
                        token_address: token.clone(),
                        tx_hash: tx_hash.to_string(),
                        block_number,
                        timestamp,
                        amount: 0,
                        transfer_type: RelationshipType::Bridge.to_string(),
                        protocol: protocol.to_string(),
                        risk_score,
                    }
                );
            }

            AmlEvent::BridgeOut {
                user,
                token,
            } => {

                rows.push(
                    AddressRelationshipRow {

                        from_address:
                        user.clone(),

                        to_address:
                        "bridge".to_string(),

                        token_address:
                        token.clone(),

                        tx_hash:
                        tx_hash.to_string(),

                        block_number,

                        timestamp,

                        amount:
                        0,

                        transfer_type:
                        RelationshipType::Bridge
                            .to_string(),

                        protocol:
                        protocol.to_string(),

                        risk_score,
                    }
                );
            }
            _ => {}
        }
    }

    rows
}
