//! Node implementation for ProzChain

use std::net::SocketAddr;
use crate::types::PeerId;

/// Configuration for connection limits
#[derive(Clone, Debug, Default)]
pub struct ConnectionLimits {
    /// Maximum number of inbound connections
    pub max_inbound: usize,
    /// Target number of outbound connections
    pub target_outbound: usize,
    /// Maximum number of peers per IP address
    pub max_peers_per_ip: usize,
}

/// Configuration for a ProzChain node
#[derive(Clone, Debug)]
pub struct NodeConfig {
    /// Type of node ("full", "validator", "light", "archive", etc.)
    pub node_type: String,
    /// Path to validator key file (if this is a validator node)
    pub validator_key_path: Option<String>,
    /// Stake amount for validator (if applicable)
    pub stake_amount: Option<u64>,
    /// Trusted validators for light clients
    pub trusted_validators: Option<Vec<String>>,
    /// Pruning strategy for blockchain data
    pub pruning_strategy: Option<String>,
    /// API server configuration
    pub api_config: Option<ApiConfig>,
    /// Addresses to listen on for incoming connections
    pub listen_addresses: Vec<String>,
    /// External addresses to advertise
    pub external_addresses: Option<Vec<String>>,
    /// Display name for this node
    pub display_name: Option<String>,
    /// Maximum number of peers to connect to
    pub max_peers: usize,
    /// Connection limits configuration
    pub connection_limits: ConnectionLimits,
}

/// Configuration for the API server
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// Whether the API server is enabled
    pub enabled: bool,
    /// Address to listen on
    pub listen_address: String,
    /// Allowed CORS domains
    pub cors_domains: Vec<String>,
    /// Rate limits for API requests
    pub rate_limit: Option<u32>,
}

/// Implementation of a ProzChain node
#[derive(Clone)]
pub struct ProzChainNode {
    /// Configuration for this node
    pub config: NodeConfig,
    /// Unique identifier for this node
    pub node_id: PeerId,
    /// Current status of the node
    pub status: NodeStatus,
}

/// Status of a node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is starting up
    Starting,
    /// Node is running normally
    Running,
    /// Node is synchronizing
    Synchronizing,
    /// Node is stopping
    Stopping,
    /// Node is stopped
    Stopped,
    /// Node encountered an error
    Error(String),
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_type: "full".to_string(),
            validator_key_path: None,
            stake_amount: None,
            trusted_validators: None,
            pruning_strategy: None,
            api_config: None,
            listen_addresses: vec!["0.0.0.0:30333".to_string()],
            external_addresses: None,
            display_name: Some("ProzChain Node".to_string()),
            max_peers: 25,
            connection_limits: ConnectionLimits {
                max_inbound: 125,
                target_outbound: 8,
                max_peers_per_ip: 1,
            },
        }
    }
}

impl ProzChainNode {
    /// Create a new ProzChain node with the given configuration
    pub fn new(config: NodeConfig) -> Result<Self, String> {
        // Generate a unique ID for this node
        let node_id = Self::generate_node_id(&config);
        
        Ok(Self {
            config,
            node_id,
            status: NodeStatus::Stopped,
        })
    }
    
    /// Generate a unique ID for this node based on its configuration
    fn generate_node_id(config: &NodeConfig) -> PeerId {
        // En una implementación real, esto generaría un ID único basado en una clave pública
        // Por ahora, usaremos un hash simple del nombre y las direcciones
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        if let Some(name) = &config.display_name {
            name.hash(&mut hasher);
        }
        
        for addr in &config.listen_addresses {
            addr.hash(&mut hasher);
        }
        
        if let Some(addrs) = &config.external_addresses {
            for addr in addrs {
                addr.hash(&mut hasher);
            }
        }
        
        // Convertir el hash u64 en un array de 32 bytes para PeerId
        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        let hash_bytes = hash.to_le_bytes();
        bytes[0..8].copy_from_slice(&hash_bytes);
        
        // Combinar con un prefijo fijo para asegurar que los IDs generados
        // sean reconocibles como de ProzChain
        bytes[8] = b'P';
        bytes[9] = b'Z';
        bytes[10] = b'C';
        bytes[11] = b'H';
        
        PeerId(bytes)
    }
    
    /// Start the node
    pub async fn start(&mut self) -> Result<(), String> {
        self.status = NodeStatus::Starting;
        
        // En una implementación real, aquí inicializaríamos todos los componentes
        // del nodo incluyendo cadena de bloques, mempool, consenso, etc.
        
        // Cambiamos el estado a Running una vez inicializado
        self.status = NodeStatus::Running;
        
        Ok(())
    }
    
    /// Stop the node
    pub async fn stop(&mut self) -> Result<(), String> {
        self.status = NodeStatus::Stopping;
        
        // En una implementación real, aquí detendríamos ordenadamente todos los componentes
        
        self.status = NodeStatus::Stopped;
        
        Ok(())
    }
    
    /// Get the current status of the node
    pub fn get_status(&self) -> NodeStatus {
        self.status.clone()
    }
    
    /// Check if this is a validator node
    pub fn is_validator(&self) -> bool {
        self.config.node_type == "validator"
    }
    
    /// Get the node's peer ID
    pub fn get_node_id(&self) -> PeerId {
        self.node_id
    }
    
    /// Get listen addresses as socket addresses
    pub fn get_listen_socket_addresses(&self) -> Vec<SocketAddr> {
        self.config.listen_addresses.iter()
            .filter_map(|addr_str| addr_str.parse().ok())
            .collect()
    }
}
