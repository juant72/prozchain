//! Network topology management

use crate::types::{DisconnectReason, PeerId};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

/// Region identifier (geographic area)
pub type RegionId = String;

/// Basic peer information
#[derive(Clone, Debug)]
pub struct Peer {
    pub id: PeerId,
    pub address: SocketAddr,
    pub connection_type: ConnectionType,
    pub region: Option<RegionId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionType {
    Inbound,
    Outbound,
}

/// Requested connection actions
#[derive(Default)]
pub struct ConnectionActions {
    pub connect: Vec<PeerId>,
    pub disconnect: Vec<(PeerId, DisconnectReason)>,
}

/// Topology configuration
#[derive(Clone, Debug)]
pub struct TopologyConfig {
    pub target_outbound: usize,
    pub max_inbound: usize,
    pub max_peers_per_ip: usize,
    pub preferred_nodes: Vec<String>,
    pub preferred_regions: Vec<String>,
}

/// Topology manager
pub struct TopologyManager {
    config: TopologyConfig,
    inbound_connections: HashMap<PeerId, SocketAddr>,
    outbound_connections: HashMap<PeerId, SocketAddr>,
    ip_connections: HashMap<std::net::IpAddr, usize>,
    preferred_peers: HashSet<PeerId>,
    peer_scores: HashMap<PeerId, f32>,
    peer_regions: HashMap<PeerId, String>,
}

impl TopologyManager {
    /// Create a new topology manager
    pub fn new(config: TopologyConfig) -> Self {
        Self {
            config,
            inbound_connections: HashMap::new(),
            outbound_connections: HashMap::new(),
            ip_connections: HashMap::new(),
            preferred_peers: HashSet::new(),
            peer_scores: HashMap::new(),
            peer_regions: HashMap::new(),
        }
    }
    
    /// Check if we can accept a new inbound connection
    pub fn can_accept_inbound(&self, addr: &SocketAddr) -> bool {
        // Check if we've reached max inbound connections
        if self.inbound_connections.len() >= self.config.max_inbound {
            return false;
        }
        
        // Check if we've reached max connections from this IP
        let ip = addr.ip();
        let ip_count = self.ip_connections.get(&ip).cloned().unwrap_or(0);
        
        if ip_count >= self.config.max_peers_per_ip {
            return false;
        }
        
        true
    }
    
    /// Register a new inbound connection
    pub fn register_inbound(&mut self, peer_id: PeerId, addr: SocketAddr) {
        self.inbound_connections.insert(peer_id, addr);
        
        // Update IP connection count
        let ip = addr.ip();
        *self.ip_connections.entry(ip).or_insert(0) += 1;
    }
    
    /// Register a new outbound connection
    pub fn register_outbound(&mut self, peer_id: PeerId, addr: SocketAddr) {
        self.outbound_connections.insert(peer_id, addr);
        
        // Update IP connection count
        let ip = addr.ip();
        *self.ip_connections.entry(ip).or_insert(0) += 1;
    }
    
    /// Remove a connection
    pub fn remove_connection(&mut self, peer_id: &PeerId) {
        // Find and remove the connection
        let addr = if let Some(addr) = self.inbound_connections.remove(peer_id) {
            addr
        } else if let Some(addr) = self.outbound_connections.remove(peer_id) {
            addr
        } else {
            return;
        };
        
        // Update IP connection count
        let ip = addr.ip();
        if let Some(count) = self.ip_connections.get_mut(&ip) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.ip_connections.remove(&ip);
            }
        }
    }
    
    /// Check if we should make more outbound connections
    pub fn needs_more_outbound(&self) -> bool {
        self.outbound_connections.len() < self.config.target_outbound
    }
    
    /// Mark a peer as preferred
    pub fn mark_preferred(&mut self, peer_id: PeerId) {
        self.preferred_peers.insert(peer_id);
    }
    
    /// Check if a peer is preferred
    pub fn is_preferred(&self, peer_id: &PeerId) -> bool {
        self.preferred_peers.contains(peer_id)
    }
    
    /// Get current peer counts
    pub fn peer_counts(&self) -> (usize, usize) {
        (self.inbound_connections.len(), self.outbound_connections.len())
    }
    
    /// Select peers for eviction when necessary
    pub fn select_eviction_candidates(&self, count: usize) -> Vec<PeerId> {
        let candidates = Vec::new();
        
        // Skip if we don't have enough peers
        if self.inbound_connections.is_empty() {
            return candidates;
        }
        
        // Exclude preferred peers
        let mut candidates: Vec<PeerId> = self.inbound_connections.keys()
            .filter(|id| !self.preferred_peers.contains(*id))
            .copied()
            .collect();
        
        // Sort by score, lowest first
        candidates.sort_by(|a, b| {
            let score_a = self.peer_scores.get(a).unwrap_or(&0.5);
            let score_b = self.peer_scores.get(b).unwrap_or(&0.5);
            score_a.partial_cmp(score_b).unwrap()
        });
        
        // Return the requested number of candidates
        candidates.truncate(count);
        candidates
    }
    
    /// Update peer score
    pub fn update_peer_score(&mut self, peer_id: PeerId, score: f32) {
        self.peer_scores.insert(peer_id, score);
    }
    
    /// Set region for a peer
    pub fn set_peer_region(&mut self, peer_id: PeerId, region: String) {
        self.peer_regions.insert(peer_id, region);
    }
    
    /// Get peers by region
    pub fn get_peers_by_region(&self, region: &str) -> Vec<PeerId> {
        self.peer_regions.iter()
            .filter_map(|(peer_id, r)| {
                if r == region {
                    Some(*peer_id)
                } else {
                    None
                }
            })
            .collect()
    }
}
