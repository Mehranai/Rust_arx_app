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