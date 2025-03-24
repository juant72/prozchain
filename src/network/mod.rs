//! Network layer implementation for ProzChain

pub mod block_propagation;
pub mod connection;
pub mod discovery;
pub mod interfaces;
pub mod message;
pub mod metrics;  // Ahora apunta al directorio metrics/ con su mod.rs
pub mod nat;
pub mod node;
pub mod propagation;
pub mod protocol_version;
pub mod security;
pub mod service;
pub mod tests;
pub mod topology;
pub mod utils;

use crate::network::service::NetworkService;
use crate::network::interfaces::NetworkServiceInterface;
use std::sync::Arc;
use std::time::Duration;

/// Create a new network service with the default configuration
pub async fn create_default_network_service() -> Result<(NetworkService, tokio::sync::mpsc::Receiver<service::NetworkResponse>), String> {
    let config = service::NetworkConfig::default();
    NetworkService::new(config).await
}

/// Initialize network metrics
pub fn init_metrics() -> Result<Arc<metrics::NetworkMetrics>, String> {
    let metrics = metrics::NetworkMetrics::new()?;
    Ok(Arc::new(metrics))
}

/// Create a health checker for the network
pub fn create_health_checker(
    metrics: Arc<metrics::NetworkMetrics>,
    network_service: impl NetworkServiceInterface + 'static,
) -> metrics::HealthChecker<impl NetworkServiceInterface> {
    let thresholds = metrics::HealthThresholds {
        min_peers: 3,
        max_latency: Duration::from_millis(500),
        min_validator_connections: 1,
        bandwidth_threshold: 1_000_000, // 1 MB/s
        message_rate_threshold: 5, // At least 5 messages per second
    };
    
    metrics::HealthChecker::new(
        metrics,
        thresholds,
        network_service,
        Duration::from_secs(60), // Check every minute
    )
}

/// Implementación de LocalNetworkServiceInterface que usa NetworkService
pub struct LocalNetworkServiceInterface {
    service: Arc<NetworkService>,
}

impl LocalNetworkServiceInterface {
    pub fn new(service: Arc<NetworkService>) -> Self {
        Self { service }
    }
}

impl NetworkServiceInterface for LocalNetworkServiceInterface {
    async fn connected_peer_count(&self) -> usize {
        self.service.connected_peer_count().await
    }

    async fn connected_validator_count(&self) -> usize {
        let peers = self.service.get_peers().await.unwrap_or_default();
        peers.iter().filter(|info| {
            info.capabilities.contains(&"VALIDATOR".to_string())
        }).count()
    }

    async fn average_peer_latency(&self) -> Duration {
        // En una implementación real, calcularía latencia real
        Duration::from_millis(100)
    }

    async fn trigger_peer_discovery(&self) {
        // En una implementación real, activaría descubrimiento de pares
        log::info!("Activando descubrimiento de pares por verificación de salud");
    }

    async fn prioritize_validator_connections(&self) {
        // En una implementación real, esto priorizaría conexiones a validadores
        log::info!("Priorizando conexiones con validadores por verificación de salud");
    }
}
