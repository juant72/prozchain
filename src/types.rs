//! Core types used across ProzChain modules

use std::fmt;

/// Unique identifier for a peer
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct PeerId(pub [u8; 32]);

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0[0..8]))
    }
}

/// Hash of a message
pub type MessageHash = [u8; 32];

/// Hash of a block
pub type BlockHash = [u8; 32];

/// Hash of a transaction
pub type TransactionHash = [u8; 32];

/// Direction of a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionDirection {
    Inbound,
    Outbound,
}

/// Reason for disconnecting a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    /// Normal disconnect - no issues
    Normal,
    /// Protocol violation
    ProtocolViolation,
    /// Peer timed out (no response)
    Timeout,
    /// Peer was banned
    PeerBanned,
    /// Too many connections
    TooManyConnections,
    /// Remote peer requested disconnect
    RemoteRequested,
    /// Internal error
    InternalError,
}

/// Protocol identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProtocolId {
    /// Peer discovery protocol
    PeerDiscovery,
    /// Block synchronization protocol
    BlockSync,
    /// Transaction propagation protocol
    TransactionPropagation,
    /// Consensus messages protocol
    ConsensusMessages,
    /// Light client synchronization protocol
    LightClientSync,
    /// Validator coordination protocol
    ValidatorCoordination,
    /// Network state query protocol
    StateQuery,
}
