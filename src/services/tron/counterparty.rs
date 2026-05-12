use std::collections::HashMap;
use crate::services::tron::aml::types::SimpleTransfer;

#[derive(Debug,Clone)]
pub struct CounterpartyRelation {
    pub address:String,
    pub counterparty:String,
    pub total_txs:u64,
    pub total_volume:u128,
    pub first_seen:u64,
    pub last_seen:u64,
}

pub fn build_counterparty_relations(
    transfers:&[SimpleTransfer],
    timestamp:u64,
)->Vec<CounterpartyRelation>
{
    let mut map=
        HashMap::<
            (String,String),
            CounterpartyRelation
        >::new();

    for t in transfers {

        let key=
            (
                t.from.clone(),
                t.to.clone()
            );

        let entry=
            map
                .entry(key)
                .or_insert(
                    CounterpartyRelation{

                        address:
                        t.from.clone(),

                        counterparty:
                        t.to.clone(),

                        total_txs:0,

                        total_volume:0,

                        first_seen:
                        timestamp,

                        last_seen:
                        timestamp,
                    }
                );

        entry.total_txs += 1;
        entry.total_volume += t.amount;
        entry.last_seen = timestamp;
    }

    map
        .into_values()
        .collect()
}