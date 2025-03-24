//! Message types and serialization for the network protocol

use crate::types::{MessageHash, ProtocolId};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Protocol identifier for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Discovery,
    BlockExchange,
    Transaction,
    Consensus,
    StateSync,
    Control,
    Identity,
}

impl Protocol {
    /// Get all available protocol types
    pub fn all() -> Vec<Self> {
        vec![
            Protocol::Discovery,
            Protocol::BlockExchange,
            Protocol::Transaction,
            Protocol::Consensus,
            Protocol::StateSync,
            Protocol::Control,
            Protocol::Identity,
        ]
    }
    
    /// Get the name of the protocol
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Discovery => "Discovery",
            Protocol::BlockExchange => "BlockExchange",
            Protocol::Transaction => "Transaction",
            Protocol::Consensus => "Consensus",
            Protocol::StateSync => "StateSync",
            Protocol::Control => "Control",
            Protocol::Identity => "Identity",
        }
    }
    
    /// Convert to numeric identifier
    pub fn as_u8(&self) -> u8 {
        match self {
            Protocol::Discovery => 0x01,
            Protocol::BlockExchange => 0x02,
            Protocol::Transaction => 0x03,
            Protocol::Consensus => 0x04,
            Protocol::StateSync => 0x05,
            Protocol::Control => 0x06,
            Protocol::Identity => 0x07,
        }
    }
    
    /// Convert from numeric identifier
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Protocol::Discovery),
            0x02 => Some(Protocol::BlockExchange),
            0x03 => Some(Protocol::Transaction),
            0x04 => Some(Protocol::Consensus),
            0x05 => Some(Protocol::StateSync),
            0x06 => Some(Protocol::Control),
            0x07 => Some(Protocol::Identity),
            _ => None,
        }
    }
}

/// Message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub protocol_id: u16,
    pub message_type: u16,
    pub length: u32,
    pub version: u8,
    pub flags: u8,
}

/// Network message
#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(protocol: Protocol, message_type: u16, payload: Vec<u8>) -> Self {
        Self {
            header: MessageHeader {
                protocol_id: protocol as u16,
                message_type,
                length: payload.len() as u32,
                version: 1,
                flags: 0,
            },
            payload,
        }
    }
    
    /// Serialize the message to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::with_capacity(12 + self.payload.len());
        
        // Serialize header
        buffer.extend_from_slice(&self.header.protocol_id.to_le_bytes());
        buffer.extend_from_slice(&self.header.message_type.to_le_bytes());
        buffer.extend_from_slice(&self.header.length.to_le_bytes());
        buffer.push(self.header.version);
        buffer.push(self.header.flags);
        
        // Add payload
        buffer.extend_from_slice(&self.payload);
        
        Ok(buffer)
    }
    
    /// Deserialize a message from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 {
            return Err("Message too short".to_string());
        }
        
        // Extract header fields
        let protocol_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        let message_type = u16::from_le_bytes([bytes[2], bytes[3]]);
        let length = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let version = bytes[8];
        let flags = bytes[9];
        
        // Check payload length
        if bytes.len() != 12 + length as usize {
            return Err(format!("Invalid message length: expected {}, got {}", 12 + length, bytes.len()));
        }
        
        // Extract payload
        let payload = bytes[12..].to_vec();
        
        Ok(Self {
            header: MessageHeader {
                protocol_id,
                message_type,
                length,
                version,
                flags,
            },
            payload,
        })
    }
}

/// Tracks recently seen messages to avoid duplication
#[derive(Debug)]
pub struct RecentMessages {
    seen: HashMap<MessageHash, Instant>,
    capacity: usize,
    ttl: Duration,
}

impl RecentMessages {
    /// Create a new RecentMessages tracker
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            seen: HashMap::with_capacity(capacity),
            capacity,
            ttl,
        }
    }
    
    /// Check if a message hash has been seen recently
    pub fn contains(&mut self, hash: &MessageHash) -> bool {
        // First cleanup expired entries
        self.cleanup();
        
        // Check if hash exists
        self.seen.contains_key(hash)
    }
    
    /// Mark a message hash as seen
    pub fn insert(&mut self, hash: MessageHash) {
        // Cleanup expired if at capacity
        if self.seen.len() >= self.capacity {
            self.cleanup();
            
            // If still at capacity, remove oldest
            if self.seen.len() >= self.capacity {
                if let Some(oldest) = self.seen.iter()
                    .min_by_key(|(_, &time)| time)
                    .map(|(hash, _)| *hash)
                {
                    self.seen.remove(&oldest);
                }
            }
        }
        
        // Insert new hash
        self.seen.insert(hash, Instant::now());
    }
    
    /// Remove expired entries
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.seen.retain(|_, time| {
            now.duration_since(*time) < self.ttl
        });
    }
}

/// Calculate a hash for a message
pub fn hash_message(message: &Message) -> MessageHash {
    // This is a simplified implementation - in reality we would use a cryptographic hash
    let mut result = [0u8; 32];
    
    // Include protocol and message type in the hash
    result[0] = (message.header.protocol_id & 0xFF) as u8;
    result[1] = (message.header.protocol_id >> 8) as u8;
    result[2] = (message.header.message_type & 0xFF) as u8;
    result[3] = (message.header.message_type >> 8) as u8;
    
    // XOR in the payload bytes
    for (i, b) in message.payload.iter().enumerate() {
        result[4 + i % 28] ^= *b;
    }
    
    result
}
