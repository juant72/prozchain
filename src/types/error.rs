use std::io;
use thiserror::Error;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Network layer specific errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Connection timeout")]
    ConnectionTimeout,
    
    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),
    
    #[error("Peer banned")]
    PeerBanned,
    
    #[error("Too many pending connections")]
    TooManyPendingConnections,
    
    #[error("Already connected")]
    AlreadyConnected,
    
    #[error("Connection limit reached")]
    ConnectionLimitReached,
    
    #[error("Invalid message format")]
    InvalidMessageFormat,
    
    #[error("Truncated message")]
    TruncatedMessage,
    
    #[error("Unknown message type")]
    UnknownMessageType,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Rate limiter not found")]
    RateLimiterNotFound,
    
    #[error("Peer address unknown")]
    PeerAddressUnknown,
    
    #[error("Invalid block reconstruction")]
    InvalidBlockReconstruction,
    
    #[error("Missing certificate")]
    MissingCertificate,
    
    #[error("Missing private key")]
    MissingPrivateKey,
    
    #[error("NAT traversal not configured")]
    NatTraversalNotConfigured,
    
    #[error("Hole punching not supported")]
    HolepunchNotSupported,
    
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for network operations
pub type Result<T> = std::result::Result<T, NetworkError>;

/// Error types for ProzChain
#[derive(Debug)]
pub enum ProzChainError {
    // Network errors
    NetworkError(String),
    ConnectionError(String),
    ProtocolError(String),
    MessageError(String),
    SecurityError(String),
    
    // Peer errors
    PeerNotFound,
    TooManyPeers,
    PeerBanned,
    
    // System errors
    IOError(std::io::Error),
    SerializationError(String),
    ConfigurationError(String),
    
    // Other errors
    UnexpectedError(String),
}

impl Display for ProzChainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ProzChainError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ProzChainError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            ProzChainError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            ProzChainError::MessageError(msg) => write!(f, "Message error: {}", msg),
            ProzChainError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            ProzChainError::PeerNotFound => write!(f, "Peer not found"),
            ProzChainError::TooManyPeers => write!(f, "Too many peers"),
            ProzChainError::PeerBanned => write!(f, "Peer is banned"),
            ProzChainError::IOError(err) => write!(f, "I/O error: {}", err),
            ProzChainError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ProzChainError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            ProzChainError::UnexpectedError(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

impl Error for ProzChainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProzChainError::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProzChainError {
    fn from(err: std::io::Error) -> Self {
        ProzChainError::IOError(err)
    }
}

/// Result type alias for ProzChain
pub type ProzChainResult<T> = std::result::Result<T, ProzChainError>;
