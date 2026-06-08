use std::sync::Arc;
use axum::{
    routing::get,
    Router,
};
use crate::config::Config;
use crate::flow_table::FlowTable;

mod flows;
mod metrics;

pub async fn serve(flow_table: Arc<FlowTable>, config: &Config) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/flows", get(flows::list_flows))
        .route("/flows/{id}", get(flows::get_flow))
        .route("/metrics", get(metrics::prometheus_metrics))
        .with_state(flow_table);

    let listener = tokio::net::TcpListener::bind(config.api_bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}
