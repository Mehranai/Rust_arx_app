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