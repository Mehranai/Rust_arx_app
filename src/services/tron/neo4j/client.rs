use neo4rs::{Graph, config, query};
use std::sync::Arc;

#[derive(Clone)]
pub struct Neo4jClient {
    pub graph: Arc<Graph>,
}

impl Neo4jClient {

    pub async fn new(
        uri: &str,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Self> {

        let cfg = config()
            .uri(uri)
            .user(username)
            .password(password)
            .db("neo4j")
            .build().expect("neo4j client configuration failure");

        let graph =
            Graph::connect(cfg).await.expect("failed to connect to neo4j(client)");

        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    pub async fn ensure_schema(&self) -> anyhow::Result<()> {
        let statements = [
            "CREATE CONSTRAINT wallet_address IF NOT EXISTS FOR (w:Wallet) REQUIRE w.address IS UNIQUE",
            "CREATE CONSTRAINT exchange_name IF NOT EXISTS FOR (e:Exchange) REQUIRE e.name IS UNIQUE",
            "CREATE INDEX transfer_tx_hash IF NOT EXISTS FOR ()-[t:TRANSFER]-() ON (t.tx_hash)",
            "CREATE INDEX wallet_exchange_role IF NOT EXISTS FOR (w:Wallet) ON (w.exchange_role)",
        ];

        for statement in statements {
            self.graph
                .run(query(statement))
                .await
                .map_err(|err| anyhow::anyhow!("{:?}", err))?;
        }

        Ok(())
    }
}
