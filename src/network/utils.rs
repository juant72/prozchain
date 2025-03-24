//! Utility functions and types for network operations

use crate::types::PeerId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Tracker for peer statistics
pub struct PeerStatistics {
    /// When the peer was first connected
    pub first_connected: Instant,
    /// Last time we had activity from this peer
    pub last_activity: Instant,
    /// Total bytes sent to this peer
    pub bytes_sent: u64,
    /// Total bytes received from this peer
    pub bytes_received: u64,
    /// Total messages sent to this peer
    pub messages_sent: u64,
    /// Total messages received from this peer
    pub messages_received: u64,
    /// Recent latency measurements (ms)
    pub recent_latencies: Vec<u64>,
}

impl PeerStatistics {
    /// Create new peer statistics
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            first_connected: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            recent_latencies: Vec::new(),
        }
    }
    
    /// Update last activity time
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Record bytes sent
    pub fn record_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
        self.update_activity();
    }
    
    /// Record bytes received
    pub fn record_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
        self.update_activity();
    }
    
    /// Record message sent
    pub fn record_message_sent(&mut self) {
        self.messages_sent += 1;
        self.update_activity();
    }
    
    /// Record message received
    pub fn record_message_received(&mut self) {
        self.messages_received += 1;
        self.update_activity();
    }
    
    /// Record latency measurement
    pub fn record_latency(&mut self, latency_ms: u64) {
        // Keep only the last 10 measurements
        if self.recent_latencies.len() >= 10 {
            self.recent_latencies.remove(0);
        }
        self.recent_latencies.push(latency_ms);
    }
    
    /// Get average latency
    pub fn average_latency(&self) -> Option<u64> {
        if self.recent_latencies.is_empty() {
            return None;
        }
        
        let sum: u64 = self.recent_latencies.iter().sum();
        Some(sum / self.recent_latencies.len() as u64)
    }
    
    /// Get uptime
    pub fn uptime(&self) -> Duration {
        Instant::now().saturating_duration_since(self.first_connected)
    }
    
    /// Get time since last activity
    pub fn idle_time(&self) -> Duration {
        Instant::now().saturating_duration_since(self.last_activity)
    }
}

/// Address to DNS name conversion helper
pub fn address_to_domain_name(addr: &SocketAddr) -> Result<String, String> {
    // In a real implementation, this might perform a reverse lookup
    // For now, just return the IP as a string
    Ok(addr.ip().to_string())
}

/// Utility to track active peer connections
pub struct PeerTracker {
    pub by_id: HashMap<PeerId, SocketAddr>,
    pub by_addr: HashMap<SocketAddr, PeerId>,
    pub statistics: HashMap<PeerId, PeerStatistics>,
}

impl PeerTracker {
    /// Create a new peer tracker
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_addr: HashMap::new(),
            statistics: HashMap::new(),
        }
    }
    
    /// Add a new peer
    pub fn add_peer(&mut self, id: PeerId, addr: SocketAddr) {
        self.by_id.insert(id, addr);
        self.by_addr.insert(addr, id);
        self.statistics.entry(id).or_insert_with(PeerStatistics::new);
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, id: &PeerId) {
        if let Some(addr) = self.by_id.remove(id) {
            self.by_addr.remove(&addr);
        }
        self.statistics.remove(id);
    }
    
    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.by_id.len()
    }
    
    /// Get peer ID by address
    pub fn get_id_by_addr(&self, addr: &SocketAddr) -> Option<&PeerId> {
        self.by_addr.get(addr)
    }
    
    /// Get peer address by ID
    pub fn get_addr_by_id(&self, id: &PeerId) -> Option<&SocketAddr> {
        self.by_id.get(id)
    }
    
    /// Get peer statistics
    pub fn get_statistics(&self, id: &PeerId) -> Option<&PeerStatistics> {
        self.statistics.get(id)
    }
    
    /// Get mutable peer statistics
    pub fn get_statistics_mut(&mut self, id: &PeerId) -> Option<&mut PeerStatistics> {
        self.statistics.get_mut(id)
    }
}
