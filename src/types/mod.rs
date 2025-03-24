//! Common type definitions used throughout ProzChain

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;  // Removed unused Hasher

pub mod error;

/// Unique identifier for a node in the network
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);  // Made public

/// Unique identifier for a peer connection
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);  // Made public

// Implementar la conversión de PeerId a u64 para usarse en BlockPropagator
impl From<PeerId> for u64 {
    fn from(peer_id: PeerId) -> Self {
        // Usar los primeros 8 bytes como u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&peer_id.0[0..8]);
        u64::from_le_bytes(bytes)
    }
}

// Implementar Display para PeerId
impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Mostrar los primeros bytes como hex
        write!(f, "{:02x}{:02x}{:02x}...", self.0[0], self.0[1], self.0[2])
    }
}

/// Hash of a block
pub type BlockHash = [u8; 32];

/// Hash of a transaction
pub type TransactionHash = [u8; 32];

/// Hash of a message
pub type MessageHash = [u8; 32];

/// Amount representation
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Amount(pub u64);  // Made public

/// Protocol identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProtocolId {
    PeerDiscovery = 0x01,
    BlockSync = 0x02,
    TransactionPropagation = 0x03,
    ConsensusMessages = 0x04,
    LightClientSync = 0x05,
    StateSync = 0x06,
}

impl ProtocolId {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x01 => Some(ProtocolId::PeerDiscovery),
            0x02 => Some(ProtocolId::BlockSync),
            0x03 => Some(ProtocolId::TransactionPropagation),
            0x04 => Some(ProtocolId::ConsensusMessages),
            0x05 => Some(ProtocolId::LightClientSync),
            0x06 => Some(ProtocolId::StateSync),
            _ => None,
        }
    }
}

/// Types of network connections
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConnectionType {
    Inbound,
    Outbound,
}

/// Direction of a connection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionDirection {
    Inbound,
    Outbound,
}

/// Reasons for disconnection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Normal,             // Graceful shutdown
    Timeout,            // No response within timeout
    ProtocolViolation,  // Peer broke protocol rules
    PeerBanned,         // Peer was banned
    TooManyPeers,       // Exceeded connection limits
    DuplicateConnection,
    IncompatibleProtocol,
    ConnectionRefused,
    NetworkError,
    Other,
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisconnectReason::Normal => write!(f, "Normal disconnect"),
            DisconnectReason::Timeout => write!(f, "Connection timeout"),
            DisconnectReason::ProtocolViolation => write!(f, "Protocol violation"),
            DisconnectReason::PeerBanned => write!(f, "Peer banned"),
            DisconnectReason::TooManyPeers => write!(f, "Too many peers"),
            DisconnectReason::DuplicateConnection => write!(f, "Duplicate connection"),
            DisconnectReason::IncompatibleProtocol => write!(f, "Incompatible protocol"),
            DisconnectReason::ConnectionRefused => write!(f, "Connection refused"),
            DisconnectReason::NetworkError => write!(f, "Network error"),
            DisconnectReason::Other => write!(f, "Other"),
        }
    }
}

/// Health status of a connection or network
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConnectionHealth {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

// Peer capabilities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PeerCapability {
    Validator,
    FullNode,
    LightClient,
    RelayNode,
    Archive,
}

/// Result of a network operation
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// Network error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    /// Connection failed
    ConnectionFailed(String),
    
    /// Connection timeout
    ConnectionTimeout,
    
    /// Peer not found
    PeerNotFound,
    
    /// Rate limit exceeded
    RateLimitExceeded,
    
    /// Message too large
    MessageTooLarge,
    
    /// Invalid message
    InvalidMessage(String),
    
    /// Protocol violation
    ProtocolViolation(String),
    
    /// IO error
    IoError(String),
    
    /// Other error
    Other(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetworkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkError::ConnectionTimeout => write!(f, "Connection timeout"),
            NetworkError::PeerNotFound => write!(f, "Peer not found"),
            NetworkError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            NetworkError::MessageTooLarge => write!(f, "Message too large"),
            NetworkError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            NetworkError::ProtocolViolation(msg) => write!(f, "Protocol violation: {}", msg),
            NetworkError::IoError(msg) => write!(f, "I/O error: {}", msg),
            NetworkError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        NetworkError::IoError(err.to_string())
    }
}

impl std::error::Error for NetworkError {}
