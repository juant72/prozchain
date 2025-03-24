//! Network metrics collection and monitoring

use crate::network::interfaces::NetworkServiceInterface;
use crate::network::message::Protocol;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, HistogramVec, IntCounter, IntGauge, Registry
};
use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Health status of the network
#[derive(Clone, Debug)]
pub struct NetworkHealthStatus {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: HealthStatus,
    pub peer_count: usize,
    pub validator_connections: usize,
    pub message_rate: f64,
    pub average_latency: Duration,
}

/// Overall health status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Critical { reason: String },
}

/// Thresholds for health checks
#[derive(Clone, Debug)]
pub struct HealthThresholds {
    pub min_peers: usize,
    pub max_latency: Duration,
    pub min_validator_connections: usize,
    pub bandwidth_threshold: usize, // bytes/second
    pub message_rate_threshold: usize, // messages/second
}

/// Health checker for network
pub struct HealthChecker<T: NetworkServiceInterface> {
    network_metrics: Arc<NetworkMetrics>,
    health_thresholds: HealthThresholds,
    check_interval: Duration,
    last_status: Option<NetworkHealthStatus>,
    network_service: T,
}

impl<T: NetworkServiceInterface> HealthChecker<T> {
    /// Create a new health checker
    pub fn new(
        network_metrics: Arc<NetworkMetrics>,
        thresholds: HealthThresholds,
        network_service: T,
        check_interval: Duration,
    ) -> Self {
        Self {
            network_metrics,
            health_thresholds: thresholds,
            check_interval,
            last_status: None,
            network_service,
        }
    }
    
    /// Run the health checker
    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            // Perform health check
            let status = self.check_health().await;
            
            // Handle status change if needed
            if self.is_status_change(&status) {
                self.handle_status_change(&status).await;
            }
            
            // Store the status
            self.last_status = Some(status);
        }
    }
    
    /// Check if this is a status change
    fn is_status_change(&self, new_status: &NetworkHealthStatus) -> bool {
        if let Some(last) = &self.last_status {
            last.status != new_status.status
        } else {
            true
        }
    }
    
    /// Check current network health
    async fn check_health(&self) -> NetworkHealthStatus {
        let mut issues = Vec::new();
        let mut critical = false;
        
        // Check peer count
        let peer_count = self.network_service.connected_peer_count().await;
        if peer_count < self.health_thresholds.min_peers {
            issues.push(format!("Insufficient peers: {} (min: {})", 
                peer_count, self.health_thresholds.min_peers));
                
            if peer_count < self.health_thresholds.min_peers / 2 {
                critical = true;
            }
        }
        
        // Check validator connections
        let validator_connections = self.network_service.connected_validator_count().await;
        if validator_connections < self.health_thresholds.min_validator_connections {
            issues.push(format!("Low validator connections: {} (min: {})",
                validator_connections, self.health_thresholds.min_validator_connections));
                
            if validator_connections == 0 {
                critical = true;
            }
        }
        
        // Check network latency
        let average_latency = self.network_service.average_peer_latency().await;
        if average_latency > self.health_thresholds.max_latency {
            issues.push(format!("High network latency: {:?} (max: {:?})",
                average_latency, self.health_thresholds.max_latency));
        }
        
        // Determine overall status
        let status = if critical {
            HealthStatus::Critical { reason: issues.join(", ") }
        } else if !issues.is_empty() {
            HealthStatus::Degraded { reason: issues.join(", ") }
        } else {
            HealthStatus::Healthy
        };
        
        NetworkHealthStatus {
            timestamp: Utc::now(),
            status,
            peer_count,
            validator_connections,
            message_rate: self.network_metrics.current_message_rate(),
            average_latency,
        }
    }
    
    /// Handle a change in health status
    async fn handle_status_change(&self, status: &NetworkHealthStatus) {
        match &status.status {
            HealthStatus::Healthy => {
                log::info!("Network health has recovered to HEALTHY");
            },
            HealthStatus::Degraded { reason } => {
                log::warn!("Network health DEGRADED: {}", reason);
                
                // Try corrective actions
                self.try_corrective_actions(status).await;
            },
            HealthStatus::Critical { reason } => {
                log::error!("Network health CRITICAL: {}", reason);
                
                // Try emergency corrective actions
                self.try_corrective_actions(status).await;
                
                // Consider alerting operators in a real system
            }
        }
    }
    
    /// Try to fix network health issues
    async fn try_corrective_actions(&self, status: &NetworkHealthStatus) {
        // Take actions based on specific issues
        if status.peer_count < self.health_thresholds.min_peers {
            // Try to discover more peers
            self.network_service.trigger_peer_discovery().await;
        }
        
        if status.validator_connections < self.health_thresholds.min_validator_connections {
            // Try to connect to more validators
            self.network_service.prioritize_validator_connections().await;
        }
    }
}

/// Metrics collection for network operations
pub struct NetworkMetrics {
    pub peer_count: IntGauge,
    pub connection_attempts: IntCounter,
    pub message_counts: HashMap<Protocol, IntCounter>,
    pub bandwidth_usage: HistogramVec,
    pub latency_histogram: HistogramVec,
    pub message_size_histogram: HistogramVec,
    pub rejected_messages: IntCounter,
    pub peer_blacklist_count: IntGauge,
    pub registry: Registry,
    
    // Internal tracking for rate calculations
    message_counter: AtomicU64,
    last_counter_reset: std::sync::Mutex<Instant>,
}

impl NetworkMetrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, String> {
        let registry = Registry::new();
        
        // Create metrics objects
        let peer_count = IntGauge::new(
            "prozchain_network_peer_count", 
            "Number of connected peers"
        ).map_err(|e| format!("Failed to create peer_count metric: {}", e))?;
        
        let connection_attempts = IntCounter::new(
            "prozchain_network_connection_attempts_total",
            "Total number of connection attempts"
        ).map_err(|e| format!("Failed to create connection_attempts metric: {}", e))?;
        
        let bandwidth_usage = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_bandwidth_bytes",
                "Bandwidth usage in bytes"
            )
            .buckets(vec![
                1000.0, 10000.0, 100000.0, 500000.0, 
                1000000.0, 5000000.0, 10000000.0
            ]),
            &["direction", "peer_type"]
        ).map_err(|e| format!("Failed to create bandwidth_usage metric: {}", e))?;
        
        let latency_histogram = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_latency_seconds",
                "Network latency in seconds"
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0
            ]),
            &["peer_type"]
        ).map_err(|e| format!("Failed to create latency_histogram metric: {}", e))?;
        
        let message_size_histogram = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_message_size_bytes",
                "Size of messages in bytes"
            )
            .buckets(vec![
                64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0
            ]),
            &["message_type"]
        ).map_err(|e| format!("Failed to create message_size_histogram metric: {}", e))?;
        
        let rejected_messages = IntCounter::new(
            "prozchain_network_rejected_messages_total",
            "Total number of rejected messages"
        ).map_err(|e| format!("Failed to create rejected_messages metric: {}", e))?;
        
        let peer_blacklist_count = IntGauge::new(
            "prozchain_network_blacklisted_peers",
            "Number of blacklisted peers"
        ).map_err(|e| format!("Failed to create peer_blacklist_count metric: {}", e))?;
        
        // Register metrics with registry
        registry.register(Box::new(peer_count.clone()))
            .map_err(|e| format!("Failed to register peer_count: {}", e))?;
        registry.register(Box::new(connection_attempts.clone()))
            .map_err(|e| format!("Failed to register connection_attempts: {}", e))?;
        registry.register(Box::new(bandwidth_usage.clone()))
            .map_err(|e| format!("Failed to register bandwidth_usage: {}", e))?;
        registry.register(Box::new(latency_histogram.clone()))
            .map_err(|e| format!("Failed to register latency_histogram: {}", e))?;
        registry.register(Box::new(message_size_histogram.clone()))
            .map_err(|e| format!("Failed to register message_size_histogram: {}", e))?;
        registry.register(Box::new(rejected_messages.clone()))
            .map_err(|e| format!("Failed to register rejected_messages: {}", e))?;
        registry.register(Box::new(peer_blacklist_count.clone()))
            .map_err(|e| format!("Failed to register peer_blacklist_count: {}", e))?;
        
        // Initialize message counters for each protocol
        let mut message_counts = HashMap::new();
        for protocol in Protocol::all() {
            let counter = IntCounter::new(
                format!("prozchain_network_{}_messages_total", protocol.name().to_lowercase()),
                format!("Total number of {} protocol messages", protocol.name())
            ).map_err(|e| format!("Failed to create message counter for {}: {}", protocol.name(), e))?;
            
            registry.register(Box::new(counter.clone()))
                .map_err(|e| format!("Failed to register message counter for {}: {}", protocol.name(), e))?;
                
            message_counts.insert(protocol, counter);
        }
        
        Ok(NetworkMetrics {
            peer_count,
            connection_attempts,
            message_counts,
            bandwidth_usage,
            latency_histogram,
            message_size_histogram,
            rejected_messages,
            peer_blacklist_count,
            registry,
            message_counter: AtomicU64::new(0),
            last_counter_reset: std::sync::Mutex::new(Instant::now()),
        })
    }
    
    /// Record a message being sent or received
    pub fn record_message(&self, protocol: Protocol, size: usize) {
        // Record message count
        if let Some(counter) = self.message_counts.get(&protocol) {
            counter.inc();
        }
        
        // Increment total message counter for rate calculation
        self.message_counter.fetch_add(1, Ordering::Relaxed);
        
        // Record message size
        self.message_size_histogram
            .with_label_values(&[protocol.name()])
            .observe(size as f64);
    }
    
    /// Record bandwidth usage
    pub fn record_bandwidth_usage(&self, bytes: usize, direction: &str, peer_type: &str) {
        self.bandwidth_usage
            .with_label_values(&[direction, peer_type])
            .observe(bytes as f64);
    }
    
    /// Record network latency
    pub fn record_latency(&self, peer_type: &str, latency: Duration) {
        self.latency_histogram
            .with_label_values(&[peer_type])
            .observe(latency.as_secs_f64());
    }
    
    /// Update peer count
    pub fn update_peer_count(&self, count: usize) {
        self.peer_count.set(count as i64);
    }
    
    /// Record connection attempt
    pub fn record_connection_attempt(&self) {
        self.connection_attempts.inc();
    }
    
    /// Record rejected message
    pub fn record_rejected_message(&self) {
        self.rejected_messages.inc();
    }
    
    /// Get average message rate (per second) over recent window
    pub fn current_message_rate(&self) -> f64 {
        let mut last_reset = self.last_counter_reset.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_reset);
        
        // Calculate rate over the time since last reset
        let count = self.message_counter.load(Ordering::Relaxed);
        let rate = if elapsed.as_secs() > 0 {
            count as f64 / elapsed.as_secs_f64()
        } else {
            // Avoid division by zero if called too quickly
            count as f64
        };
        
        // Reset counters every 60 seconds to provide a moving average
        if elapsed > Duration::from_secs(60) {
            self.message_counter.store(0, Ordering::Relaxed);
            *last_reset = now;
        }
        
        rate
    }
    
    /// Update blacklisted peer count
    pub fn update_blacklist_count(&self, count: usize) {
        self.peer_blacklist_count.set(count as i64);
    }
}
