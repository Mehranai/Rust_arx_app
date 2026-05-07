use std::collections::{HashMap, HashSet};

pub const ZERO_ADDRESS: &str =
    "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb";

// --------------------------------------------------
// CORE TRANSFER MODEL
// --------------------------------------------------

#[derive(Debug, Clone)]
pub struct SimpleTransfer {
    pub token: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
}

// --------------------------------------------------
// AML EVENT TYPES
// --------------------------------------------------

#[derive(Debug, Clone)]
pub enum AmlEvent {
    Swap {
        user: String,
        token_in: String,
        token_out: String,
    },

    BridgeIn {
        user: String,
        token: String,
    },

    BridgeOut {
        user: String,
        token: String,
    },

    Mint {
        user: String,
        token: String,
    },

    Burn {
        user: String,
        token: String,
    },
}

// --------------------------------------------------
// NET FLOW ENGINE
// address -> token -> signed delta
// --------------------------------------------------

pub fn compute_net_flows(
    transfers: &[SimpleTransfer],
) -> HashMap<String, HashMap<String, i128>> {

    let mut flows: HashMap<
        String,
        HashMap<String, i128>,
    > = HashMap::new();

    for t in transfers {

        let amount =
            i128::try_from(t.amount).unwrap_or(i128::MAX);

        //
        // sender loses
        //
        flows
            .entry(t.from.clone())
            .or_default()
            .entry(t.token.clone())
            .and_modify(|v| *v -= amount)
            .or_insert(-amount);

        //
        // receiver gains
        //
        flows
            .entry(t.to.clone())
            .or_default()
            .entry(t.token.clone())
            .and_modify(|v| *v += amount)
            .or_insert(amount);
    }

    flows
}

// --------------------------------------------------
// SWAP DETECTION
// --------------------------------------------------

pub fn detect_swaps(
    transfers: &[SimpleTransfer],
) -> Vec<AmlEvent> {

    let flows = compute_net_flows(transfers);

    let mut events = Vec::new();

    let mut dedup = HashSet::new();

    for (address, token_map) in flows {

        if address == ZERO_ADDRESS {
            continue;
        }

        let mut sent = Vec::new();
        let mut received = Vec::new();

        for (token, delta) in token_map {

            if delta < 0 {
                sent.push(token.clone());
            }

            if delta > 0 {
                received.push(token.clone());
            }
        }

        for token_in in &sent {
            for token_out in &received {

                if token_in == token_out {
                    continue;
                }

                let key = format!(
                    "{}:{}:{}",
                    address,
                    token_in,
                    token_out
                );

                if dedup.contains(&key) {
                    continue;
                }

                dedup.insert(key);

                events.push(
                    AmlEvent::Swap {
                        user: address.clone(),
                        token_in: token_in.clone(),
                        token_out: token_out.clone(),
                    }
                );
            }
        }
    }

    events
}

// --------------------------------------------------
// MINT / BURN DETECTION
// --------------------------------------------------

pub fn detect_mints_and_burns(
    transfers: &[SimpleTransfer],
) -> Vec<AmlEvent> {

    let mut events = Vec::new();

    for t in transfers {

        //
        // Mint
        //
        if t.from == ZERO_ADDRESS {

            events.push(
                AmlEvent::Mint {
                    user: t.to.clone(),
                    token: t.token.clone(),
                }
            );
        }

        //
        // Burn
        //
        if t.to == ZERO_ADDRESS {

            events.push(
                AmlEvent::Burn {
                    user: t.from.clone(),
                    token: t.token.clone(),
                }
            );
        }
    }

    events
}

// --------------------------------------------------
// BRIDGE DETECTION
// --------------------------------------------------


pub fn detect_bridges(
    transfers: &[SimpleTransfer],
) -> Vec<AmlEvent> {

    let flows = compute_net_flows(transfers);

    let mut events = Vec::new();

    for (address, token_map) in flows {

        if address == ZERO_ADDRESS {
            continue;
        }

        for (token, delta) in token_map {

            //
            // bridge in
            //
            if delta > 0
                && transfers.iter().any(|t|
                t.from == ZERO_ADDRESS
                    && t.to == address
                    && t.token == token
            )
            {
                events.push(
                    AmlEvent::BridgeIn {
                        user: address.clone(),
                        token: token.clone(),
                    }
                );
            }

            //
            // bridge out
            //
            if delta < 0
                && transfers.iter().any(|t|
                t.to == ZERO_ADDRESS
                    && t.from == address
                    && t.token == token
            )
            {
                events.push(
                    AmlEvent::BridgeOut {
                        user: address.clone(),
                        token: token.clone(),
                    }
                );
            }
        }
    }

    events
}

//
// --------------------------------------------------
// HELPERS
// --------------------------------------------------
//

pub fn has_swap(events: &[AmlEvent]) -> bool {

    events.iter().any(|e|
        matches!(e, AmlEvent::Swap { .. })
    )
}

pub fn has_bridge(events: &[AmlEvent]) -> bool {

    events.iter().any(|e|
        matches!(
            e,
            AmlEvent::BridgeIn { .. }
            | AmlEvent::BridgeOut { .. }
        )
    )
}