//! Connection management

use crate::types::{ConnectionDirection, DisconnectReason, PeerId};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};

/// Connection configuration
#[derive(Clone, Debug)]
pub struct ConnectionConfig {
    pub handshake_timeout: Duration,
    pub max_pending_connections: usize,
    pub tls_config: Option<TlsConfig>,
    pub enable_0rtt: bool,
}

/// TLS configuration (placeholder)
#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub certificate_path: std::path::PathBuf,
    pub private_key_path: std::path::PathBuf,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Handshaking,
    Connected,
    ShuttingDown,
    Disconnected,
}

/// Connection limits
#[derive(Clone, Debug)]
pub struct ConnectionLimits {
    pub max_inbound_connections: usize,
    pub max_outbound_connections: usize,
    pub max_connections_per_ip: usize,
    pub max_pending_handshakes: usize,
    pub handshake_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        ConnectionLimits {
            max_inbound_connections: 128,
            max_outbound_connections: 32,
            max_connections_per_ip: 5,
            max_pending_handshakes: 10,
            handshake_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60 * 10), // 10 minutes
        }
    }
}

/// Connection information
#[derive(Debug)]
pub struct Connection {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub direction: ConnectionDirection,
    pub state: ConnectionState,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub user_agent: String,
    pub protocol_version: u32,
    pub message_tx: mpsc::Sender<Vec<u8>>,
}

/// Connection manager
pub struct ConnectionManager {
    config: ConnectionConfig,
    limits: ConnectionLimits,
    connections: HashMap<PeerId, Arc<Connection>>,
    // The streams would be handled by tokio tasks
    connecting: HashMap<SocketAddr, Instant>,
    handshaking: HashMap<SocketAddr, Instant>,
    ip_connections: HashMap<std::net::IpAddr, usize>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: ConnectionConfig, limits: ConnectionLimits) -> Self {
        Self {
            config,
            limits,
            connections: HashMap::new(),
            connecting: HashMap::new(),
            handshaking: HashMap::new(),
            ip_connections: HashMap::new(),
        }
    }
    
    /// Handle a new inbound connection
    pub fn handle_inbound_connection(&mut self, stream: TcpStream, addr: SocketAddr) -> Result<(), String> {
        // Check connection limits
        if self.connections.len() >= self.limits.max_inbound_connections {
            return Err("Too many connections".to_string());
        }
        
        // Check IP limits
        let ip = addr.ip();
        let ip_count = self.ip_connections.get(&ip).cloned().unwrap_or(0);
        if ip_count >= self.limits.max_connections_per_ip {
            return Err("Too many connections from this IP".to_string());
        }
        
        // Check if we're already connecting to this address
        if self.connecting.contains_key(&addr) || self.handshaking.contains_key(&addr) {
            return Err("Already connecting to this address".to_string());
        }
        
        // Add to handshaking list
        self.handshaking.insert(addr, Instant::now());
        
        // Start handshake process in a background task
        self.start_inbound_handshake(stream, addr);
        
        Ok(())
    }
    
    /// Start the handshake process for an inbound connection
    fn start_inbound_handshake(&self, stream: TcpStream, addr: SocketAddr) {
        // Clone what we need for the async task
        let handshake_timeout = self.config.handshake_timeout;
        let tls_config = self.config.tls_config.clone();
        
        // Spawn handshake task
        tokio::spawn(async move {
            // Set up timeout
            let handshake_future = Self::perform_inbound_handshake(stream, addr, tls_config);
            let handshake_result = tokio::time::timeout(handshake_timeout, handshake_future).await;
            
            match handshake_result {
                Ok(Ok((peer_id, protocol_version, user_agent))) => {
                    // Handshake successful
                    log::info!("Handshake completed with peer {} at {}", peer_id, addr);
                    
                    // In a real implementation, we would register the new connection
                    // and set up message handling
                },
                Ok(Err(e)) => {
                    log::warn!("Handshake with {} failed: {}", addr, e);
                },
                Err(_) => {
                    log::warn!("Handshake with {} timed out", addr);
                }
            }
        });
    }
    
    /// Perform the inbound handshake
    async fn perform_inbound_handshake(
        _stream: TcpStream, 
        _addr: SocketAddr,
        _tls_config: Option<TlsConfig>
    ) -> Result<(PeerId, u32, String), String> {
        // In a real implementation, this would:
        // 1. Upgrade to TLS if configured
        // 2. Send version information
        // 3. Receive peer's version information
        // 4. Validate compatibility
        // 5. Exchange peer IDs
        
        // For now, just return a placeholder result
        let peer_id = generate_temporary_peer_id();
        let protocol_version = 1;
        let user_agent = "prozchain-rust/0.1.0".to_string();
        
        Ok((peer_id, protocol_version, user_agent))
    }
    
    /// Establish an outbound connection
    pub async fn establish_outbound_connection(&mut self, addr: SocketAddr) -> Result<Connection, String> {
        // Check connection limits
        if self.connections.len() >= self.limits.max_outbound_connections {
            return Err("Too many outbound connections".to_string());
        }
        
        // Check IP limits
        let ip = addr.ip();
        let ip_count = self.ip_connections.get(&ip).cloned().unwrap_or(0);
        if ip_count >= self.limits.max_connections_per_ip {
            return Err("Too many connections to this IP".to_string());
        }
        
        // Check if we're already connecting to this address
        if self.connecting.contains_key(&addr) {
            return Err("Already connecting to this address".to_string());
        }
        
        // Add to connecting list
        self.connecting.insert(addr, Instant::now());
        
        // Try to connect
        let stream = match tokio::time::timeout(
            self.config.handshake_timeout,
            TcpStream::connect(addr)
        ).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                self.connecting.remove(&addr);
                return Err(format!("Connection failed: {}", e));
            },
            Err(_) => {
                self.connecting.remove(&addr);
                return Err("Connection timed out".to_string());
            }
        };
        
        // Now perform handshake
        let handshake_result = self.perform_outbound_handshake(stream, addr).await;
        
        // Remove from connecting list
        self.connecting.remove(&addr);
        
        match handshake_result {
            Ok(connection) => {
                // Update IP connection count
                *self.ip_connections.entry(ip).or_insert(0) += 1;
                
                // In a real implementation, we would store the connection
                
                Ok(connection)
            },
            Err(e) => Err(e),
        }
    }
    
    /// Perform the outbound handshake
    async fn perform_outbound_handshake(&self, _stream: TcpStream, addr: SocketAddr) -> Result<Connection, String> {
        // In a real implementation, this would:
        // 1. Upgrade to TLS if configured
        // 2. Send version information
        // 3. Receive peer's version information
        // 4. Validate compatibility
        // 5. Exchange peer IDs
        
        // For now, just return a placeholder connection
        let peer_id = generate_temporary_peer_id();
        
        // Create message channel
        let (message_tx, _message_rx) = mpsc::channel(100);
        
        Ok(Connection {
            peer_id,
            address: addr,
            direction: ConnectionDirection::Outbound,
            state: ConnectionState::Connected,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            user_agent: "unknown".to_string(),
            protocol_version: 1,
            message_tx,
        })
    }
    
    /// Disconnect a peer
    pub fn disconnect(&mut self, peer_id: PeerId, reason: DisconnectReason) {
        if let Some(connection) = self.connections.remove(&peer_id) {
            let ip = connection.address.ip();
            
            // Update IP connection count
            if let Some(count) = self.ip_connections.get_mut(&ip) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.ip_connections.remove(&ip);
                }
            }
            
            // Log disconnect
            log::debug!(
                "Disconnected from peer {} at {} with reason {:?}",
                peer_id, connection.address, reason
            );
            
            // In a real implementation, we would notify the peer and clean up resources
        }
    }
    
    /// Get the number of connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
    
    /// Get the number of inbound connections
    pub fn inbound_connection_count(&self) -> usize {
        self.connections.values()
            .filter(|conn| conn.direction == ConnectionDirection::Inbound)
            .count()
    }
    
    /// Get the number of outbound connections
    pub fn outbound_connection_count(&self) -> usize {
        self.connections.values()
            .filter(|conn| conn.direction == ConnectionDirection::Outbound)
            .count()
    }
    
    /// Check if we're connected to a peer
    pub fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.connections.contains_key(peer_id)
    }
    
    /// Get a list of connected peers
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connections.keys().copied().collect()
    }
    
    /// Clean up stale connections
    pub fn clean_up_stale_connections(&mut self) {
        let now = Instant::now();
        
        // Check for stale handshakes
        self.handshaking.retain(|addr, time| {
            if now.duration_since(*time) > self.limits.handshake_timeout {
                log::debug!("Removing stale handshake with {}", addr);
                false
            } else {
                true
            }
        });
        
        // Check for stale connections
        let stale_peers: Vec<_> = self.connections.iter()
            .filter_map(|(id, conn)| {
                if now.duration_since(conn.last_activity) > self.limits.idle_timeout {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        
        for peer_id in stale_peers {
            self.disconnect(peer_id, DisconnectReason::Timeout);
        }
    }
}

/// Generate a temporary peer ID
fn generate_temporary_peer_id() -> PeerId {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rand::Rng::fill(&mut rng, &mut bytes);
    PeerId(bytes)
}
