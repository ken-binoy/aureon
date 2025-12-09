use crate::metrics::Metrics;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

/// Health check response
#[derive(Debug, Serialize, Clone)]
pub struct HealthCheck {
    pub status: String,
    pub timestamp: u64,
    pub chain_height: u64,
    pub peers_connected: u64,
    pub mempool_size: u64,
}

impl HealthCheck {
    pub fn healthy(chain_height: u64, peers_connected: u64, mempool_size: u64) -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            chain_height,
            peers_connected,
            mempool_size,
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            status: "unhealthy".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            chain_height: 0,
            peers_connected: 0,
            mempool_size: 0,
        }
    }
}

/// Monitoring metrics summary
#[derive(Debug, Serialize)]
pub struct MetricsSummary {
    pub blocks_produced: u64,
    pub blocks_received: u64,
    pub transactions_processed: u64,
    pub consensus_rounds: u64,
    pub peers_connected: u64,
    pub chain_height: u64,
}

/// Create monitoring router with health checks and metrics endpoints
pub fn monitoring_router(metrics: Arc<Metrics>) -> Router {
    Router::new()
        .route("/health", get(health_check).with_state(metrics.clone()))
        .route("/metrics", get(prometheus_metrics).with_state(metrics.clone()))
        .route("/metrics/summary", get(metrics_summary).with_state(metrics))
}

/// Health check endpoint
async fn health_check(
    State(metrics): State<Arc<Metrics>>,
) -> Result<Json<HealthCheck>, (StatusCode, String)> {
    // Get current metrics values
    let chain_height = metrics.chain_height.get() as u64;
    let peers_connected = metrics.peers_connected.get() as u64;
    let mempool_size = metrics.mempool_size.get() as u64;

    // Consider healthy if chain height > 0 or peers connected > 0
    let health = if chain_height > 0 || peers_connected > 0 {
        HealthCheck::healthy(chain_height, peers_connected, mempool_size)
    } else {
        HealthCheck::unhealthy()
    };

    Ok(Json(health))
}

/// Prometheus metrics endpoint
async fn prometheus_metrics(
    State(metrics): State<Arc<Metrics>>,
) -> Result<Response, (StatusCode, String)> {
    match metrics.export() {
        Ok(output) => Ok((
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            output,
        )
            .into_response()),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to export metrics".to_string(),
        )),
    }
}

/// Metrics summary endpoint
async fn metrics_summary(
    State(metrics): State<Arc<Metrics>>,
) -> Result<Json<MetricsSummary>, (StatusCode, String)> {
    let summary = MetricsSummary {
        blocks_produced: metrics
            .blocks_produced
            .get()
            .try_into()
            .unwrap_or_default(),
        blocks_received: metrics.blocks_received.get().try_into().unwrap_or_default(),
        transactions_processed: metrics
            .transactions_processed
            .get()
            .try_into()
            .unwrap_or_default(),
        consensus_rounds: metrics.consensus_rounds.get().try_into().unwrap_or_default(),
        peers_connected: metrics.peers_connected.get() as u64,
        chain_height: metrics.chain_height.get() as u64,
    };

    Ok(Json(summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_creation() {
        let health = HealthCheck::healthy(10, 3, 5);
        assert_eq!(health.status, "healthy");
        assert_eq!(health.chain_height, 10);
        assert_eq!(health.peers_connected, 3);
        assert_eq!(health.mempool_size, 5);
    }

    #[test]
    fn test_health_check_unhealthy() {
        let health = HealthCheck::unhealthy();
        assert_eq!(health.status, "unhealthy");
        assert_eq!(health.chain_height, 0);
    }

    #[test]
    fn test_metrics_summary_creation() {
        let summary = MetricsSummary {
            blocks_produced: 100,
            blocks_received: 200,
            transactions_processed: 1000,
            consensus_rounds: 50,
            peers_connected: 5,
            chain_height: 100,
        };
        assert_eq!(summary.blocks_produced, 100);
        assert_eq!(summary.peers_connected, 5);
    }
}
