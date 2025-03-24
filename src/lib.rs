//! ProzChain blockchain library
//! 
//! This crate provides the core functionality for the ProzChain blockchain,
//! including networking, consensus, and storage components.

pub mod network;
pub mod types;

/// Re-export key components for easier access
pub use network::service::NetworkService;
pub use network::message::Message;
pub use network::node::ProzChainNode;
pub use types::PeerId;
pub use types::{ConnectionDirection, DisconnectReason, ProtocolId}; // Re-exportamos los tipos de red

/// Initialize the library
pub fn init() -> Result<(), String> {
    // Set up logging if not already configured
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    if let Err(e) = env_logger::try_init() {
        return Err(format!("Failed to initialize logger: {}", e));
    }
    
    Ok(())
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
