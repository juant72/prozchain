//! Core types used across ProzChain modules
//! This module contains common types used throughout the ProzChain system

mod error;
mod network;

pub use error::Error;
pub use network::*;

use std::fmt;
use serde::{Deserialize, Serialize};

/// Unique identifier for a peer
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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
