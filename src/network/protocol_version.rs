//! Protocol versioning and capability negotiation

use crate::types::ProtocolId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Protocol version using semantic versioning
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // Major version must match exactly
        if self.major != other.major {
            return false;
        }
        
        // Our minor version must be equal or greater
        if self.minor < other.minor {
            return false;
        }
        
        true
    }
    
    /// Get string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Feature flags for protocol negotiations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureFlag {
    CompactBlocks,
    CompactTransactions,
    FastSync,
    HeaderVerification,
    Compression,
    Encryption,
    PriorityTransactions,
    GrapheneBlockSupport,
    AnchorSync,
}

/// Protocol capabilities advertised by a node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    pub supported_protocols: HashMap<ProtocolId, ProtocolVersion>,
    pub features: HashSet<FeatureFlag>,
}

/// The result of protocol negotiation
#[derive(Debug, Clone)]
pub struct NegotiatedProtocols {
    pub protocols: HashMap<ProtocolId, ProtocolVersion>,
    pub features: HashSet<FeatureFlag>,
}

/// Handles protocol version negotiation
pub struct ProtocolNegotiator {
    pub local_capabilities: ProtocolCapabilities,
    pub min_compatible_version: HashMap<ProtocolId, ProtocolVersion>,
}

impl ProtocolNegotiator {
    /// Create a new protocol negotiator
    pub fn new(
        local_capabilities: ProtocolCapabilities,
        min_compatible_version: HashMap<ProtocolId, ProtocolVersion>,
    ) -> Self {
        Self {
            local_capabilities,
            min_compatible_version,
        }
    }

    /// Negotiate compatible protocols and features
    pub fn negotiate(&self, remote_capabilities: &ProtocolCapabilities) -> NegotiatedProtocols {
        let mut result = NegotiatedProtocols {
            protocols: HashMap::new(),
            features: HashSet::new(),
        };
        
        // Find compatible protocols
        for (protocol, local_version) in &self.local_capabilities.supported_protocols {
            if let Some(remote_version) = remote_capabilities.supported_protocols.get(protocol) {
                // Check if the remote version is compatible with our minimum
                if let Some(min_version) = self.min_compatible_version.get(protocol) {
                    if !min_version.is_compatible_with(remote_version) {
                        continue;
                    }
                }
                
                // Use the lower version for compatibility
                let negotiated_version = ProtocolVersion {
                    major: local_version.major,
                    minor: std::cmp::min(local_version.minor, remote_version.minor),
                    patch: std::cmp::min(local_version.patch, remote_version.patch),
                };
                
                result.protocols.insert(*protocol, negotiated_version);
            }
        }
        
        // Find common features
        for feature in &self.local_capabilities.features {
            if remote_capabilities.features.contains(feature) {
                result.features.insert(*feature);
            }
        }
        
        result
    }
}

/// Helper function to create a default set of capabilities
pub fn default_capabilities(node_type: &str) -> ProtocolCapabilities {
    let mut supported_protocols = HashMap::new();
    let mut features = HashSet::new();
    
    // Add base protocols supported by all nodes
    supported_protocols.insert(ProtocolId::PeerDiscovery, ProtocolVersion::new(1, 0, 0));
    
    // Add node-type specific protocols and features
    match node_type {
        "full" | "validator" | "archive" => {
            supported_protocols.insert(ProtocolId::BlockSync, ProtocolVersion::new(1, 0, 0));
            supported_protocols.insert(ProtocolId::TransactionPropagation, ProtocolVersion::new(1, 0, 0));
            
            features.insert(FeatureFlag::Compression);
            features.insert(FeatureFlag::CompactTransactions);
            
            if node_type == "validator" {
                supported_protocols.insert(ProtocolId::ConsensusMessages, ProtocolVersion::new(1, 0, 0));
                features.insert(FeatureFlag::PriorityTransactions);
            }
            
            if node_type == "archive" {
                features.insert(FeatureFlag::AnchorSync);
            }
        },
        "light" => {
            supported_protocols.insert(ProtocolId::LightClientSync, ProtocolVersion::new(1, 0, 0));
            features.insert(FeatureFlag::HeaderVerification);
            features.insert(FeatureFlag::FastSync);
        },
        _ => {}
    }
    
    ProtocolCapabilities {
        supported_protocols,
        features,
    }
}
