use neo4rs::{query};
use super::client::Neo4jClient;

pub async fn upsert_wallet(
    neo4j: &Neo4jClient,
    address: &str,
) -> anyhow::Result<()> {

    let q = query(
        "
        MERGE (w:Wallet {
            address: $address
        })
        "
    )
        .param("address", address);

    neo4j
        .graph
        .run(q)
        .await.expect("failed to execute neo4j (Nodes 1)");

    Ok(())
}

pub async fn upsert_wallet_with_metadata(
    neo4j: &Neo4jClient,
    address: &str,
    exchange_name: Option<&str>,
    exchange_role: Option<&str>,
    confidence: Option<f32>,
) -> anyhow::Result<()> {
    let node_type = if exchange_name.is_some() {
        "exchange_wallet"
    } else {
        "wallet"
    };

    let q = query(
        "
        MERGE (w:Wallet { address: $address })
        SET w.node_type = $node_type,
            w.exchange_name = $exchange_name,
            w.exchange_role = $exchange_role,
            w.exchange_confidence_bps = $confidence_bps
        "
    )
        .param("address", address)
        .param("node_type", node_type)
        .param("exchange_name", exchange_name.unwrap_or(""))
        .param("exchange_role", exchange_role.unwrap_or(""))
        .param(
            "confidence_bps",
            (confidence.unwrap_or(0.0) * 10_000.0) as i64,
        );

    neo4j
        .graph
        .run(q)
        .await
        .map_err(|err| anyhow::anyhow!("{:?}", err))?;

    if let Some(exchange) = exchange_name {
        if !exchange.is_empty() {
            upsert_exchange(neo4j, address, exchange).await?;
        }
    }

    Ok(())
}

pub async fn upsert_exchange(
    neo4j: &Neo4jClient,
    address: &str,
    exchange: &str,
) -> anyhow::Result<()> {

    let q = query(
        "
        MERGE (e:Exchange {
            name: $exchange
        })

        MERGE (w:Wallet {
            address: $address
        })

        MERGE (w)-[:BELONGS_TO]->(e)
        "
    )
        .param("address", address)
        .param("exchange", exchange);

    neo4j
        .graph
        .run(q)
        .await.expect("failed to execute neo4j (Nodes 2)");

    Ok(())
}
