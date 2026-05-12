use neo4rs::{Graph, config};
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
}