use axum::{Router, routing::get};
use crate::handlers::{health, status, tron_graph};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/status", get(status::status))
        .route(
            "/tron/wallet/{address}/graph",
            get(tron_graph::tron_wallet_graph),
        )
}
