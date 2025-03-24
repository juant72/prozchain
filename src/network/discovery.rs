//! Peer discovery mechanisms

use crate::types::PeerId;
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Bootstrap configuration
#[derive(Clone, Debug)]
pub struct BootstrapConfig {
    pub bootstrap_nodes: Vec<String>,
    pub dns_seeds: Vec<String>,
    pub enable_local_discovery: bool,
    pub static_peers: Vec<String>,
    pub dns_lookup_interval: Duration,
}

/// Peer information
#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub id: PeerId,
    pub address: SocketAddr,
    pub protocol_version: u32,
    pub user_agent: String,
    pub capabilities: Vec<String>,
    pub service_bits: u64,
}

/// Local peer record
#[derive(Clone, Debug)]
struct LocalPeerRecord {
    first_seen: Instant,
    last_seen: Instant,
    address: SocketAddr,
    protocol_version: u32,
    user_agent: String,
    source: PeerSource,
}

/// Source of peer information
#[derive(Clone, Debug, PartialEq, Eq)]
enum PeerSource {
    Bootstrap,
    DNSSeed,
    PeerExchange,
    LocalDiscovery,
    ManuallyAdded,
    Incoming,
}

/// Peer discovery module
pub struct PeerDiscovery {
    config: BootstrapConfig,
    known_peers: HashMap<PeerId, LocalPeerRecord>,
    attempted_peers: HashSet<SocketAddr>,
    banned_peers: HashSet<SocketAddr>,
    last_dns_lookup: Option<Instant>,
    local_address: Option<SocketAddr>,
    mdns_discovery: Option<MdnsDiscovery>,
    peer_db_path: Option<std::path::PathBuf>,
}

/// mDNS discovery (placeholder)
struct MdnsDiscovery {
    enabled: bool,
}

impl PeerDiscovery {
    /// Create a new peer discovery instance
    pub fn new(config: BootstrapConfig) -> Self {
        Self {
            config,
            known_peers: HashMap::new(),
            attempted_peers: HashSet::new(),
            banned_peers: HashSet::new(),
            last_dns_lookup: None,
            local_address: None,
            mdns_discovery: Some(MdnsDiscovery { enabled: false }),
            peer_db_path: None,
        }
    }
    
    /// Bootstrap the peer discovery
    pub async fn bootstrap(&mut self) -> Result<Vec<PeerInfo>, String> {
        let mut discovered_peers = Vec::new();
        
        // Clone bootstrap nodes to avoid borrowing self immutably during iteration
        let bootstrap_nodes = self.config.bootstrap_nodes.clone();
        
        // Add bootstrap nodes
        for node in bootstrap_nodes {
            if let Ok(peers) = self.resolve_peer_address(&node).await {
                for (addr, source) in peers {
                    let peer_id = self::generate_temporary_peer_id(&addr);
                    let peer_info = PeerInfo {
                        id: peer_id,
                        address: addr,
                        protocol_version: 1,
                        user_agent: "unknown".to_string(),
                        capabilities: vec!["FULL_NODE".to_string()],
                        service_bits: 1,
                    };
                    
                    self.add_peer(peer_id, addr, 1, "unknown", source);
                    discovered_peers.push(peer_info);
                }
            }
        }
        
        // Query DNS seeds
        if discovered_peers.len() < 10 {
            for seed in self.config.dns_seeds.clone() {
                if let Ok(peers) = self.query_dns_seed(&seed).await {
                    for (addr, source) in peers {
                        let peer_id = self::generate_temporary_peer_id(&addr);
                        let peer_info = PeerInfo {
                            id: peer_id,
                            address: addr,
                            protocol_version: 1,
                            user_agent: "unknown".to_string(),
                            capabilities: vec!["FULL_NODE".to_string()],
                            service_bits: 1,
                        };
                        
                        self.add_peer(peer_id, addr, 1, "unknown", source);
                        discovered_peers.push(peer_info);
                    }
                }
            }
            
            self.last_dns_lookup = Some(Instant::now());
        }
        
        // Try local discovery if enabled
        if self.config.enable_local_discovery {
            if let Some(mdns) = &mut self.mdns_discovery {
                if mdns.enabled {
                    // Would perform mDNS discovery here
                    log::debug!("Local discovery via mDNS not implemented yet");
                }
            }
        }
        
        // Load saved peers
        if let Some(db_path) = &self.peer_db_path {
            if let Ok(loaded_peers) = self.load_saved_peers(db_path).await {
                for (peer_id, addr, proto_ver, user_agent) in loaded_peers {
                    let peer_info = PeerInfo {
                        id: peer_id,
                        address: addr,
                        protocol_version: proto_ver,
                        user_agent: user_agent.clone(),
                        capabilities: vec!["FULL_NODE".to_string()],
                        service_bits: 1,
                    };
                    
                    self.add_peer(peer_id, addr, proto_ver, &user_agent, PeerSource::ManuallyAdded);
                    discovered_peers.push(peer_info);
                }
            }
        }
        
        log::info!("Discovered {} peers during bootstrap", discovered_peers.len());
        Ok(discovered_peers)
    }
    
    /// Add a peer to known peers
    fn add_peer(&mut self, peer_id: PeerId, addr: SocketAddr, protocol_version: u32, user_agent: &str, source: PeerSource) {
        let now = Instant::now();
        
        let record = self.known_peers.entry(peer_id).or_insert(LocalPeerRecord {
            first_seen: now,
            last_seen: now,
            address: addr,
            protocol_version,
            user_agent: user_agent.to_string(),
            source: source.clone(),
        });
        
        // Update existing record
        record.last_seen = now;
        if source != PeerSource::Incoming {
            record.address = addr;
        }
        record.protocol_version = protocol_version;
        record.user_agent = user_agent.to_string();
        
        // If this is a higher priority source, update the source
        if source_priority(&source) > source_priority(&record.source) {
            record.source = source;
        }
    }
    
    /// Resolve peer address from string
    async fn resolve_peer_address(&self, address: &str) -> Result<Vec<(SocketAddr, PeerSource)>, String> {
        let source = PeerSource::Bootstrap;
        
        // Try to resolve the address
        match address.to_socket_addrs() {
            Ok(addrs) => {
                let result: Vec<_> = addrs.map(|addr| (addr, source.clone())).collect();
                if result.is_empty() {
                    Err(format!("Could not resolve address: {}", address))
                } else {
                    Ok(result)
                }
            }
            Err(e) => Err(format!("Failed to parse address {}: {}", address, e)),
        }
    }
    
    /// Query a DNS seed for peers
    async fn query_dns_seed(&self, seed: &str) -> Result<Vec<(SocketAddr, PeerSource)>, String> {
        let source = PeerSource::DNSSeed;
        
        // Try DNS resolution
        match tokio::net::lookup_host(format!("{}:30333", seed)).await {
            Ok(addrs) => {
                let result: Vec<_> = addrs.map(|addr| (addr, source.clone())).collect();
                if result.is_empty() {
                    Err(format!("No addresses found for DNS seed: {}", seed))
                } else {
                    Ok(result)
                }
            }
            Err(e) => Err(format!("Failed to resolve DNS seed {}: {}", seed, e)),
        }
    }
    
    /// Load peers from persistent storage
    async fn load_saved_peers(&self, _db_path: &std::path::Path) -> Result<Vec<(PeerId, SocketAddr, u32, String)>, String> {
        // In a real implementation, this would load peers from disk
        // For now, return an empty list
        Ok(Vec::new())
    }
    
    /// Find more peers if needed
    pub async fn find_more_peers(&mut self, target_count: usize) -> Result<Vec<PeerInfo>, String> {
        // If we already have enough peers, no need to look for more
        if self.known_peers.len() >= target_count {
            return Ok(Vec::new());
        }
        
        // Check if it's time to do another DNS lookup
        let should_query_dns = if let Some(last_lookup) = self.last_dns_lookup {
            last_lookup.elapsed() > self.config.dns_lookup_interval
        } else {
            true
        };
        
        let mut discovered_peers = Vec::new();
        
        // Query DNS seeds if needed
        if should_query_dns {
            for seed in self.config.dns_seeds.clone() {
                if let Ok(peers) = self.query_dns_seed(&seed).await {
                    for (addr, source) in peers {
                        // Skip already attempted addresses
                        if self.attempted_peers.contains(&addr) || self.banned_peers.contains(&addr) {
                            continue;
                        }
                        
                        let peer_id = self::generate_temporary_peer_id(&addr);
                        let peer_info = PeerInfo {
                            id: peer_id,
                            address: addr,
                            protocol_version: 1,
                            user_agent: "unknown".to_string(),
                            capabilities: vec!["FULL_NODE".to_string()],
                            service_bits: 1,
                        };
                        
                        self.add_peer(peer_id, addr, 1, "unknown", source);
                        discovered_peers.push(peer_info);
                    }
                }
            }
            
            self.last_dns_lookup = Some(Instant::now());
        }
        
        // Return newly discovered peers
        Ok(discovered_peers)
    }
    
    /// Mark a peer as attempted
    pub fn mark_attempted(&mut self, addr: SocketAddr) {
        self.attempted_peers.insert(addr);
    }
    
    /// Mark a peer as banned
    pub fn mark_banned(&mut self, addr: SocketAddr) {
        self.banned_peers.insert(addr);
    }
    
    /// Set local listening address
    pub fn set_local_address(&mut self, addr: SocketAddr) {
        self.local_address = Some(addr);
    }
    
    /// Set peer database path
    pub fn set_peer_db_path(&mut self, path: std::path::PathBuf) {
        self.peer_db_path = Some(path);
    }
    
    /// Get a list of all known peers
    pub fn get_known_peers(&self) -> Vec<PeerInfo> {
        self.known_peers.iter().map(|(id, record)| {
            PeerInfo {
                id: *id,
                address: record.address,
                protocol_version: record.protocol_version,
                user_agent: record.user_agent.clone(),
                capabilities: vec!["FULL_NODE".to_string()], // Placeholder
                service_bits: 1, // Placeholder
            }
        }).collect()
    }
}

/// Get priority of peer source
fn source_priority(source: &PeerSource) -> u8 {
    match source {
        PeerSource::ManuallyAdded => 5,
        PeerSource::Incoming => 4,
        PeerSource::LocalDiscovery => 3,
        PeerSource::Bootstrap => 2,
        PeerSource::PeerExchange => 1,
        PeerSource::DNSSeed => 0,
    }
}

/// Generate a temporary peer ID from address
fn generate_temporary_peer_id(addr: &SocketAddr) -> PeerId {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    addr.hash(&mut hasher);
    let addr_hash = hasher.finish();
    
    // Convert hash to bytes
    let bytes = addr_hash.to_le_bytes();
    
    // Create peer ID with hash bytes repeated
    let mut id_bytes = [0u8; 32];
    for i in 0..4 {
        id_bytes[i*8..(i+1)*8].copy_from_slice(&bytes);
    }
    
    PeerId(id_bytes)
}
