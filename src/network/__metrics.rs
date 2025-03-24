//! Network metrics collection and monitoring

use crate::network::message::Protocol;
use crate::network::service::NetworkServiceInterface;
use crate::types::PeerId;
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, HistogramVec, IntCounter, IntGauge, Registry
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    registry: Registry,
}

impl NetworkMetrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, String> {
        let registry = Registry::new();
        
        // Create peer count gauge
        let peer_count = IntGauge::new(
            "prozchain_network_peer_count", 
            "Number of connected peers"
        ).map_err(|e| format!("Failed to create peer_count metric: {}", e))?;
        
        // Create connection attempts counter
        let connection_attempts = IntCounter::new(
            "prozchain_network_connection_attempts_total",
            "Total number of connection attempts"
        ).map_err(|e| format!("Failed to create connection_attempts metric: {}", e))?;
        
        // Create bandwidth usage histogram
        let bandwidth_usage = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_bandwidth_bytes",
                "Bandwidth usage in bytes"
            ).buckets(vec![
                1_000.0, 10_000.0, 100_000.0, 500_000.0, 
                1_000_000.0, 5_000_000.0, 10_000_000.0
            ]),
            &["direction", "peer_type"]
        ).map_err(|e| format!("Failed to create bandwidth_usage metric: {}", e))?;
        
        // Create latency histogram
        let latency_histogram = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_latency_seconds",
                "Network latency in seconds"
            ).buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
            ]),
            &["peer_type"]
        ).map_err(|e| format!("Failed to create latency_histogram metric: {}", e))?;
        
        // Create message size histogram
        let message_size_histogram = HistogramVec::new(
            HistogramOpts::new(
                "prozchain_network_message_size_bytes",
                "Size of messages in bytes"
            ).buckets(vec![
                64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0
            ]),
            &["message_type"]
        ).map_err(|e| format!("Failed to create message_size_histogram metric: {}", e))?;
        
        // Create rejected messages counter
        let rejected_messages = IntCounter::new(
            "prozchain_network_rejected_messages_total",
            "Total number of rejected messages"
        ).map_err(|e| format!("Failed to create rejected_messages metric: {}", e))?;
        
        // Create blacklisted peers gauge
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
                format!("prozchain_network_messages_{}_total", protocol.name().to_lowercase()),
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
        })
    }
    
    /// Record a message being sent or received
    pub fn record_message(&self, protocol: Protocol, size: usize) {
        // Record message count
        if let Some(counter) = self.message_counts.get(&protocol) {
            counter.inc();
        }
        
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
    
    /// Update blacklisted peer count
    pub fn update_blacklist_count(&self, count: usize) {
        self.peer_blacklist_count.set(count as i64);
    }
    
    /// Get average message rate (per second) over recent window
    pub fn current_message_rate(&self) -> f64 {
        // In a real implementation, this would calculate based on recent history
        // For now, return a placeholder
        10.0
    }
    
    /// Get registry for exposing metrics
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Health checker for network
pub struct HealthChecker<T: NetworkServiceInterface> {
    network_metrics: Arc<NetworkMetrics>,
    health_thresholds: HealthThresholds,
    check_interval: Duration,
    last_status: Option<NetworkHealthStatus>,
    network_service: T,
}

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

/// Interface to network service
pub trait NetworkServiceInterface {
    // This would be a simplified interface to the network service
    // For now, just placeholder methods
    fn connected_peer_count(&self) -> usize;
    fn connected_validator_count(&self) -> usize;
    fn average_peer_latency(&self) -> Duration;
    fn trigger_peer_discovery(&self);
    fn prioritize_validator_connections(&self);
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
    
    /// Start the health checker
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
        let peer_count = self.network_service.connected_peer_count();
        if peer_count < self.health_thresholds.min_peers {
            issues.push(format!("Insufficient peers: {} (min: {})", 
                peer_count, self.health_thresholds.min_peers));
                
            if peer_count < self.health_thresholds.min_peers / 2 {
                critical = true;
            }
        }
        
        // Check validator connections
        let validator_connections = self.network_service.connected_validator_count();
        if validator_connections < self.health_thresholds.min_validator_connections {
            issues.push(format!("Low validator connections: {} (min: {})",
                validator_connections, self.health_thresholds.min_validator_connections));
                
            if validator_connections == 0 {
                critical = true;
            }
        }
        
        // Check network latency
        let average_latency = self.network_service.average_peer_latency();
        if average_latency > self.health_thresholds.max_latency {
            issues.push(format!("High network latency: {:?} (max: {:?})",
                average_latency, self.health_thresholds.max_latency));
        }
        
        // Check message rate
        let message_rate = self.network_metrics.current_message_rate();
        if message_rate < self.health_thresholds.message_rate_threshold as f64 {
            issues.push(format!("Low message rate: {:.2} msgs/sec (min: {})",
                message_rate, self.health_thresholds.message_rate_threshold));
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
            timestamp: chrono::Utc::now(),
            status,
            peer_count,
            validator_connections,
            message_rate,
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
            self.network_service.trigger_peer_discovery();
        }
        
        if status.validator_connections < self.health_thresholds.min_validator_connections {
            // Try to connect to more validators
            self.network_service.prioritize_validator_connections();
        }
    }
}

/// Performance analyzer for network optimization
pub struct PerformanceAnalyzer {
    metrics_service: Arc<NetworkMetrics>,
    analyzer_config: AnalyzerConfig,
    network_service: NetworkServiceInterface,
    results_cache: HashMap<String, AnalysisResult>,
    last_analysis: Option<Instant>,
}

/// Configuration for the analyzer
pub struct AnalyzerConfig {
    pub analysis_window: Duration,
    pub peer_sample_size: usize,
    pub analysis_triggers: Vec<AnalysisTrigger>,
    pub automatic_optimizations: bool,
    pub min_peers: usize,
    pub bandwidth_warning_threshold: usize,
}

/// Conditions that trigger analysis
pub enum AnalysisTrigger {
    Scheduled { interval: Duration },
    OnDemand,
    MetricThreshold { metric: String, threshold: f64 },
}

/// Result of a performance analysis
pub struct AnalysisResult {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub recommendations: Vec<NetworkRecommendation>,
    pub metrics_snapshot: Option<MetricsSnapshot>,
}

/// Snapshot of metrics for analysis
pub struct MetricsSnapshot {
    pub peer_count: usize,
    pub message_rates: HashMap<Protocol, f64>,
    pub bandwidth_usage: f64,
    pub average_latency: Duration,
}

/// A network optimization recommendation
pub struct NetworkRecommendation {
    pub action: String,
    pub expected_benefit: String,
    pub priority: RecommendationPriority,
    pub automation_possible: bool,
}

/// Priority of a recommendation
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new(
        metrics: Arc<NetworkMetrics>,
        config: AnalyzerConfig,
        network_service: NetworkServiceInterface,
    ) -> Self {
        Self {
            metrics_service: metrics,
            analyzer_config: config,
            network_service,
            results_cache: HashMap::new(),
            last_analysis: None,
        }
    }
    
    /// Analyze network performance
    pub async fn analyze_network_performance(&mut self) -> AnalysisResult {
        // Collect current metrics
        let metrics_snapshot = self.collect_metrics().await;
        
        // Analyze peer connectivity
        let peer_analysis = self.analyze_peer_connectivity(&metrics_snapshot).await;
        
        // Analyze message propagation
        let propagation_analysis = self.analyze_message_propagation(&metrics_snapshot).await;
        
        // Analyze bandwidth usage
        let bandwidth_analysis = self.analyze_bandwidth_usage(&metrics_snapshot).await;
        
        // Generate optimization recommendations
        let recommendations = self.generate_recommendations(
            &peer_analysis,
            &propagation_analysis,
            &bandwidth_analysis
        ).await;
        
        // Apply automatic optimizations if enabled
        if self.analyzer_config.automatic_optimizations {
            self.apply_optimizations(&recommendations).await.unwrap_or_else(|e| {
                log::error!("Failed to apply optimizations: {}", e);
            });
        }
        
        // Update analysis time
        self.last_analysis = Some(Instant::now());
        
        // Prepare final report
        let result = AnalysisResult {
            timestamp: chrono::Utc::now(),
            recommendations,
            metrics_snapshot: Some(metrics_snapshot),
        };
        
        // Cache the result
        self.results_cache.insert("latest".to_string(), result.clone());
        
        result
    }
    
    /// Collect current metrics
    async fn collect_metrics(&self) -> MetricsSnapshot {
        // In a real implementation, this would gather actual metrics
        // For now, just return placeholder data
        MetricsSnapshot {
            peer_count: 5,
            message_rates: HashMap::new(),
            bandwidth_usage: 50_000.0, // 50 KB/sec
            average_latency: Duration::from_millis(100),
        }
    }
    
    /// Analyze peer connectivity
    async fn analyze_peer_connectivity(&self, metrics: &MetricsSnapshot) -> PeerAnalysis {
        let mut analysis = PeerAnalysis {
            total_peers: metrics.peer_count,
            peer_recommendations: Vec::new(),
            geographic_distribution: HashMap::new(),
        };
        
        // Generate recommendations
        if analysis.total_peers < self.analyzer_config.min_peers {
            analysis.peer_recommendations.push(NetworkRecommendation {
                action: format!("Increase connection count from {} to at least {}", 
                    analysis.total_peers, self.analyzer_config.min_peers),
                expected_benefit: "Improved network resilience and message propagation".to_string(),
                priority: RecommendationPriority::High,
                automation_possible: true,
            });
        }
        
        analysis
    }
    
    /// Analyze message propagation
    async fn analyze_message_propagation(&self, _metrics: &MetricsSnapshot) -> PropagationAnalysis {
        // In a real implementation, this would analyze message propagation
        PropagationAnalysis {
            average_block_propagation: Duration::from_millis(250),
            average_transaction_propagation: Duration::from_millis(100),
        }
    }
    
    /// Analyze bandwidth usage
    async fn analyze_bandwidth_usage(&self, metrics: &MetricsSnapshot) -> BandwidthAnalysis {
        // In a real implementation, this would analyze bandwidth usage patterns
        BandwidthAnalysis {
            outbound_bandwidth: metrics.bandwidth_usage as usize,
            inbound_bandwidth: (metrics.bandwidth_usage * 0.8) as usize,
            peer_bandwidth_distribution: HashMap::new(),
        }
    }
    
    /// Generate recommendations
    async fn generate_recommendations(
        &self,
        peer_analysis: &PeerAnalysis,
        propagation_analysis: &PropagationAnalysis,
        bandwidth_analysis: &BandwidthAnalysis
    ) -> Vec<NetworkRecommendation> {
        let mut recommendations = Vec::new();
        
        // Peer connectivity recommendations
        recommendations.extend(peer_analysis.peer_recommendations.clone());
        
        // Bandwidth optimization recommendations
        if bandwidth_analysis.outbound_bandwidth > self.analyzer_config.bandwidth_warning_threshold {
            recommendations.push(NetworkRecommendation {
                action: format!("Reduce outbound bandwidth from {} bytes/sec by enabling more aggressive message batching", 
                    bandwidth_analysis.outbound_bandwidth),
                expected_benefit: "Lower network resource utilization".to_string(),
                priority: RecommendationPriority::Medium,
                automation_possible: true,
            });
        }
        
        // Message propagation recommendations
        if propagation_analysis.average_block_propagation > Duration::from_secs(1) {
            recommendations.push(NetworkRecommendation {
                action: "Enable compact block relay".to_string(),
                expected_benefit: "Faster block propagation".to_string(),
                priority: RecommendationPriority::High,
                automation_possible: true,
            });
        }
        
        recommendations
    }
    
    /// Apply recommended optimizations
    async fn apply_optimizations(&self, recommendations: &[NetworkRecommendation]) -> Result<(), String> {
        for recommendation in recommendations {
            if recommendation.automation_possible {
                log::info!("Automatically applying optimization: {}", recommendation.action);
                // In a real implementation, this would apply the optimization
            }
        }
        
        Ok(())
    }
}

/// Analysis of peer connectivity
pub struct PeerAnalysis {
    pub total_peers: usize,
    pub peer_recommendations: Vec<NetworkRecommendation>,
    pub geographic_distribution: HashMap<String, usize>,
}

/// Analysis of message propagation
pub struct PropagationAnalysis {
    pub average_block_propagation: Duration,
    pub average_transaction_propagation: Duration,
}

/// Analysis of bandwidth usage
pub struct BandwidthAnalysis {
    pub outbound_bandwidth: usize,
    pub inbound_bandwidth: usize,
    pub peer_bandwidth_distribution: HashMap<PeerId, f64>,
}
