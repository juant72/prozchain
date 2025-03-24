//! Network service interfaces

use std::time::Duration;
use crate::types::PeerId;

/// Interface to network service for metrics and health checking
pub trait NetworkServiceInterface: Send + Sync {
    /// Get number of connected peers
    async fn connected_peer_count(&self) -> usize;
    
    /// Get number of connected validator peers
    async fn connected_validator_count(&self) -> usize;
    
    /// Get average network latency to peers
    async fn average_peer_latency(&self) -> Duration;
    
    /// Trigger peer discovery process
    async fn trigger_peer_discovery(&self);
    
    /// Prioritize connections to validator nodes
    async fn prioritize_validator_connections(&self);
}

/// Interface for block propagation
pub trait BlockPropagationInterface: Send + Sync {
    /// Propagate a block to the network
    async fn propagate_block(&self, block_data: Vec<u8>) -> Result<(), String>;
    
    /// Handle block announcement
    async fn handle_block_announcement(&self, peer_id: PeerId, data: Vec<u8>) -> Result<(), String>;
}

/// Interface for transaction propagation
pub trait TransactionPropagationInterface: Send + Sync {
    /// Propagate a transaction to the network
    async fn propagate_transaction(&self, tx_data: Vec<u8>) -> Result<(), String>;
    
    /// Handle transaction announcement
    async fn handle_transaction_announcement(&self, peer_id: PeerId, data: Vec<u8>) -> Result<(), String>;
}

/// Interface for peer discovery
pub trait PeerDiscoveryInterface: Send + Sync {
    /// Discover new peers
    async fn discover_peers(&self) -> Result<Vec<crate::network::discovery::PeerInfo>, String>;
    
    /// Register a new peer that was discovered through other means
    async fn register_peer(&self, peer_info: crate::network::discovery::PeerInfo);
    
    /// Get known peers
    async fn get_known_peers(&self) -> Vec<crate::network::discovery::PeerInfo>;
}

/// Interface for connection management
pub trait ConnectionManagerInterface: Send + Sync {
    /// Connect to a peer
    async fn connect_to_peer(&self, address: std::net::SocketAddr) -> Result<PeerId, String>;
    
    /// Disconnect from a peer
    async fn disconnect_peer(&self, peer_id: &PeerId, reason: crate::types::DisconnectReason);
    
    /// Check if connected to a peer
    async fn is_connected_to(&self, peer_id: &PeerId) -> bool;
    
    /// Get connection count
    async fn connection_count(&self) -> usize;
}
