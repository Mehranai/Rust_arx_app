use neo4rs::{query};
use super::client::Neo4jClient;

pub async fn create_transfer_edge(
    neo4j: &Neo4jClient,

    from: &str,
    to: &str,

    tx_hash: &str,

    token: &str,

    amount: &str,

    risk_score: u8,

    transfer_type: &str,
) -> anyhow::Result<()> {

    let q = query(
        "
        MERGE (a:Wallet {
            address: $from
        })

        MERGE (b:Wallet {
            address: $to
        })

        CREATE (a)-[:TRANSFER {
            tx_hash: $tx_hash,
            token: $token,
            amount: $amount,
            risk_score: $risk_score,
            transfer_type: $transfer_type
        }]->(b)
        "
    )
        .param("from", from)
        .param("to", to)
        .param("tx_hash", tx_hash)
        .param("token", token)
        .param("amount", amount)
        .param("risk_score", risk_score as i64)
        .param("transfer_type", transfer_type);

    neo4j
        .graph
        .run(q)
        .await.expect("failed to execute neo4j (Edges)");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn merge_transfer_edge(
    neo4j: &Neo4jClient,
    from: &str,
    to: &str,
    tx_hash: &str,
    token: &str,
    amount: &str,
    block_number: u64,
    timestamp: u64,
    risk_score: u8,
    transfer_type: &str,
    protocol: &str,
) -> anyhow::Result<()> {
    let edge_id = format!(
        "{}:{}:{}:{}:{}",
        tx_hash, from, to, token, transfer_type
    );

    let q = query(
        "
        MERGE (a:Wallet { address: $from })
        MERGE (b:Wallet { address: $to })
        MERGE (a)-[t:TRANSFER { id: $edge_id }]->(b)
        SET t.tx_hash = $tx_hash,
            t.token = $token,
            t.amount = $amount,
            t.block_number = $block_number,
            t.timestamp = $timestamp,
            t.risk_score = $risk_score,
            t.transfer_type = $transfer_type,
            t.protocol = $protocol
        "
    )
        .param("from", from)
        .param("to", to)
        .param("edge_id", edge_id)
        .param("tx_hash", tx_hash)
        .param("token", token)
        .param("amount", amount)
        .param("block_number", block_number as i64)
        .param("timestamp", timestamp as i64)
        .param("risk_score", risk_score as i64)
        .param("transfer_type", transfer_type)
        .param("protocol", protocol);

    neo4j
        .graph
        .run(q)
        .await
        .map_err(|err| anyhow::anyhow!("{:?}", err))?;

    Ok(())
}
