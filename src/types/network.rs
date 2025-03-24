//! Network-specific types

use serde::{Deserialize, Serialize};

/// Direction of a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionDirection {
    Inbound,
    Outbound,
}

/// Reason for disconnecting a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
