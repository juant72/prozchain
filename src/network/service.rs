use crate::network::connection::{ConnectionConfig, ConnectionManager};
use crate::network::discovery::{PeerDiscovery, PeerInfo};
use crate::network::message::{Message, Protocol};
use crate::network::nat::NatTraversal;
use crate::network::node::{NodeConfig, ProzChainNode};
use crate::network::propagation::BroadcastManager;
use crate::network::topology::TopologyManager;
use crate::types::{DisconnectReason, PeerId};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

// Add imports for new features
use crate::network::protocol_version::{ProtocolCapabilities, ProtocolNegotiator, NegotiatedProtocols, ProtocolVersion};
use crate::network::security::{SybilProtection, AddressRestrictionLevel, DoSProtection, ResourceType};
use crate::network::block_propagation::{BlockAnnouncement, BlockPropagator, BlockPreference, Block};
use crate::network::interfaces::NetworkServiceInterface as NetworkServiceInterfaceTrait;
use crate::network::metrics::NetworkMetrics;
use crate::types::ProtocolId;

/// Configuration for the network service
#[derive(Clone)]
pub struct NetworkConfig {
    pub node_config: NodeConfig,
    pub listen_addresses: Vec<String>,
    pub bootstrap_nodes: Vec<String>,
    pub dns_seeds: Vec<String>,
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub ping_interval: Duration,
    pub peer_exchange_interval: Duration,
    pub enable_upnp: bool,
    pub enable_nat_traversal: bool,
    pub stun_servers: Vec<String>,
    pub whitelist: Option<HashSet<String>>,
    pub blacklist: Option<HashSet<String>>,
}

// Adding Default implementation
impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            node_config: Default::default(),
            listen_addresses: vec!["0.0.0.0:30333".to_string()],
            bootstrap_nodes: vec![
                "bootstrap1.prozchain.io:30333".to_string(),
                "bootstrap2.prozchain.io:30333".to_string(),
            ],
            dns_seeds: vec![
                "seed1.prozchain.io".to_string(),
                "seed2.prozchain.io".to_string(),
            ],
            max_peers: 25,
            connection_timeout: Duration::from_secs(10),
            ping_interval: Duration::from_secs(60),
            peer_exchange_interval: Duration::from_secs(300),
            enable_upnp: true,
            enable_nat_traversal: true,
            stun_servers: vec!["stun.prozchain.io:3478".to_string()],
            whitelist: None,
            blacklist: None,
        }
    }
}

/// Message for the network service
pub enum NetworkMessage {
    Connect(SocketAddr),
    Disconnect(PeerId, DisconnectReason),
    PeerDiscovered(PeerInfo),
    SendMessage(PeerId, Message),
    Broadcast(Protocol, Message),
    GetPeers,
    Shutdown,
}

/// Status of the network service
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// Response from the network service
pub enum NetworkResponse {
    PeerList(Vec<PeerInfo>),
    ConnectionResult(Result<(), String>),
    MessageSent(Result<(), String>),
    BroadcastResult(Result<(), String>),
    Shutdown,
}

/// Main network service
pub struct NetworkService {
    config: NetworkConfig,
    node: ProzChainNode,
    status: Arc<RwLock<NetworkStatus>>,
    connection_manager: Arc<RwLock<ConnectionManager>>,
    peer_discovery: Arc<RwLock<PeerDiscovery>>,
    topology_manager: Arc<RwLock<TopologyManager>>,
    broadcast_manager: Arc<RwLock<BroadcastManager>>,
    nat_traversal: Arc<RwLock<NatTraversal>>,
    #[allow(dead_code)]
    _message_tx: mpsc::Sender<NetworkMessage>,
    message_rx: tokio::sync::Mutex<Option<mpsc::Receiver<NetworkMessage>>>,
    response_tx: mpsc::Sender<NetworkResponse>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    shutdown_signal: tokio::sync::broadcast::Sender<()>,

    // New fields for additional features
    metrics: Option<Arc<NetworkMetrics>>,
    block_propagator: Option<Arc<RwLock<BlockPropagator>>>,
    peer_block_preferences: Arc<RwLock<HashMap<PeerId, BlockPreference>>>,
    protocol_negotiator: Arc<RwLock<ProtocolNegotiator>>,
    sybil_protection: Arc<RwLock<SybilProtection>>,
    dos_protection: Arc<RwLock<DoSProtection>>,
}

impl NetworkService {
    /// Create a new network service
    pub async fn new(config: NetworkConfig) -> Result<(Self, mpsc::Receiver<NetworkResponse>), String> {
        // Create message channels
        let (message_tx, message_rx) = mpsc::channel(100);
        let (response_tx, response_rx) = mpsc::channel(100);

        // Create shutdown signal
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        // Create node
        let node = ProzChainNode::new(config.node_config.clone())?;

        // Initialize components
        let connection_manager = Arc::new(RwLock::new(
            ConnectionManager::new(
                ConnectionConfig {
                    handshake_timeout: config.connection_timeout,
                    max_pending_connections: 10,
                    tls_config: None, // Would be set in production
                    enable_0rtt: false,
                },
                Default::default(), // Connection limits would be set in production
            )
        ));

        // Other components would be initialized here
        let peer_discovery = Arc::new(RwLock::new(PeerDiscovery::default()));
        let topology_manager = Arc::new(RwLock::new(TopologyManager::default()));
        let broadcast_manager = Arc::new(RwLock::new(BroadcastManager::default()));
        let nat_traversal = Arc::new(RwLock::new(NatTraversal::default()));

        // Add protocol version capabilities
        let local_capabilities = crate::network::protocol_version::default_capabilities(&config.node_config.node_type);
        let mut min_versions = HashMap::new();
        min_versions.insert(ProtocolId::PeerDiscovery, ProtocolVersion::new(1, 0, 0));
        min_versions.insert(ProtocolId::BlockSync, ProtocolVersion::new(1, 0, 0));

        let protocol_negotiator = ProtocolNegotiator::new(
            local_capabilities, 
            min_versions
        );

        // Set up security mechanisms
        let sybil_protection = SybilProtection::new(
            8, // min_outbound
            2, // max_per_ip
            10, // max_per_subnet
            20, // max_per_asn
            AddressRestrictionLevel::LimitPerSubnet,
        );

        let whitelist = config.whitelist.clone().unwrap_or_default()
            .iter()
            .filter_map(|addr_str| {
                addr_str.parse::<IpAddr>().ok()
            })
            .collect();

        // Create connection manager interface for DoS protection
        let conn_interface = crate::network::security::ConnectionManagerInterface::new();

        let dos_protection = DoSProtection::new(conn_interface, whitelist);

        // Initialize metrics if enabled
        #[cfg(feature = "metrics")]
        let metrics = Some(Arc::new(NetworkMetrics::new()?));
        #[cfg(not(feature = "metrics"))]
        let metrics = None;

        let service = NetworkService {
            config,
            node,
            status: Arc::new(RwLock::new(NetworkStatus::Stopped)),
            connection_manager,
            peer_discovery,
            topology_manager,
            broadcast_manager,
            nat_traversal,
            _message_tx: message_tx,
            message_rx: tokio::sync::Mutex::new(Some(message_rx)),
            response_tx,
            peers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signal: shutdown_tx,
            metrics,
            block_propagator: None,
            peer_block_preferences: Arc::new(RwLock::new(HashMap::new())),
            protocol_negotiator: Arc::new(RwLock::new(protocol_negotiator)),
            sybil_protection: Arc::new(RwLock::new(sybil_protection)),
            dos_protection: Arc::new(RwLock::new(dos_protection)),
        };

        Ok((service, response_rx))
    }

    /// Start the network service
    pub async fn start(&self) -> Result<(), String> {
        // Update status
        *self.status.write().await = NetworkStatus::Starting;

        // Start listening for incoming connections
        let listeners = self.start_listeners().await?;

        // Discover external IP and NAT type
        if self.config.enable_nat_traversal {
            self.nat_traversal.write().await.discover_nat_type().await?;
        }

        // Set up UPnP port forwarding if enabled
        if self.config.enable_upnp {
            for listener in &listeners {
                let local_addr = listener.local_addr().map_err(|e| e.to_string())?;
                let local_port = local_addr.port();
                self.nat_traversal.write().await.try_upnp_port_mapping(local_port).await?;
            }
        }

        // Bootstrap initial peer connections
        self.bootstrap_peer_connections().await?;

        // Set up block propagation mechanisms
        self.setup_block_propagation().await?;

        // Start maintenance tasks
        self.start_maintenance_tasks();

        // Start message handling loop
        self.start_message_handler().await;

        // Set up metrics
        if let Some(metrics) = &self.metrics {
            // Update initial peer count
            let peer_count = self.peers.read().await.len();
            metrics.update_peer_count(peer_count);

            // Create and start health checker if metrics are enabled
            let network_interface = LocalNetworkServiceInterface::new(
                Arc::new(self.clone())
            );

            let thresholds = crate::network::metrics::HealthThresholds {
                min_peers: 3,
                max_latency: Duration::from_millis(500),
                min_validator_connections: 1,
                bandwidth_threshold: 1_000_000, // 1 MB/s
                message_rate_threshold: 5,
            };

            let mut health_checker = crate::network::metrics::HealthChecker::new(
                metrics.clone(),
                thresholds,
                network_interface,
                Duration::from_secs(60),
            );

            // Start health checker in a background task
            tokio::spawn(async move {
                health_checker.run().await;
            });
        }

        // Update status
        *self.status.write().await = NetworkStatus::Running;

        Ok(())
    }

    /// Start listening for incoming connections
    async fn start_listeners(&self) -> Result<Vec<Arc<TcpListener>>, String> {
        let mut listeners = Vec::new();

        for address in &self.config.listen_addresses {
            match TcpListener::bind(address).await {
                Ok(listener) => {
                    let listener = Arc::new(listener);
                    log::info!("Listening for connections on {}", address);

                    // Spawn task to handle incoming connections
                    let connection_manager = self.connection_manager.clone();
                    let message_tx = self._message_tx.clone();
                    let shutdown_signal = self.shutdown_signal.subscribe();

                    // Clone the listener for the task
                    let task_listener = Arc::clone(&listener);

                    tokio::spawn(async move {
                        Self::handle_incoming_connections(
                            task_listener,
                            connection_manager,
                            message_tx,
                            shutdown_signal
                        ).await;
                    });

                    // Now we can add the original listener to our list
                    listeners.push(listener);
                }
                Err(e) => {
                    log::warn!("Failed to bind to {}: {}", address, e);
                }
            }
        }

        if listeners.is_empty() {
            return Err("Failed to bind to any listen addresses".to_string());
        }

        Ok(listeners)
    }

    /// Handle incoming connections on a listener
    async fn handle_incoming_connections(
        listener: Arc<TcpListener>,
        connection_manager: Arc<RwLock<ConnectionManager>>,
        message_tx: mpsc::Sender<NetworkMessage>,
        mut shutdown_signal: tokio::sync::broadcast::Receiver<()>,
    ) {
        loop {
            // Check for shutdown signal
            let accept_future = (*listener).accept();
            let result = tokio::select! {
                result = accept_future => result,
                _ = shutdown_signal.recv() => break,
            };

            match result {
                Ok((stream, addr)) => {
                    log::debug!("Accepted connection from {}", addr);

                    // Handle the connection
                    let mut connection_manager = connection_manager.write().await;
                    match connection_manager.handle_inbound_connection(stream, addr) {
                        Ok(()) => {
                            // Connection will be handled by connection manager
                        }
                        Err(e) => {
                            log::debug!("Rejected connection from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Error accepting connection: {}", e);
                }
            }
        }

        log::info!("Connection listener stopped");
    }

    /// Bootstrap initial peer connections
    async fn bootstrap_peer_connections(&self) -> Result<(), String> {
        // Perform initial peer discovery
        let discovered = self.peer_discovery.write().await.bootstrap().await?;

        // Store discovered peers
        for peer in discovered {
            self.peers.write().await.insert(peer.id, peer.clone());

            // Try to connect to some of the discovered peers
            if self.peers.read().await.len() < self.config.max_peers {
                if let Err(e) = self.connect_to_peer(peer.address).await {
                    log::debug!("Failed to connect to {}: {}", peer.address, e);
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Connect to a peer by address
    async fn connect_to_peer(&self, address: SocketAddr) -> Result<(), String> {
        // Check if an incoming connection is allowed (avoid Sybil attacks)
        if !self.is_connection_allowed(&address).await {
            return Err("Connection not allowed due to security restrictions".to_string());
        }

        // Establish outbound connection
        let connection = self.connection_manager.write().await
            .establish_outbound_connection(address).await?;

        // Store peer info in our database
        let peer_info = PeerInfo {
            id: connection.peer_id,
            address: connection.address,
            protocol_version: 1, // Would come from handshake
            user_agent: "prozchain-rust/0.1.0".to_string(), // Would come from handshake
            capabilities: vec!["FULL_NODE".to_string()], // Would come from handshake
            service_bits: 1, // Would come from handshake
        };

        // Record the connection for security tracking
        self.record_connection(address).await;

        self.peers.write().await.insert(connection.peer_id, peer_info.clone());

        // Notify about new peer
        if let Err(e) = self._message_tx.send(NetworkMessage::PeerDiscovered(peer_info)).await {
            log::warn!("Failed to send peer discovered message: {}", e);
        }

        Ok(())
    }

    /// Start periodic maintenance tasks
    fn start_maintenance_tasks(&self) {
        // Clone required components
        let connection_manager = self.connection_manager.clone();
        let topology_manager = self.topology_manager.clone();
        let peer_discovery = self.peer_discovery.clone();
        let peers = self.peers.clone();
        let message_tx = self._message_tx.clone();
        let mut shutdown_signal = self.shutdown_signal.subscribe();
        let ping_interval = self.config.ping_interval;

        // Spawn maintenance task
        tokio::spawn(async move {
            let mut interval = time::interval(ping_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform maintenance
                        Self::perform_maintenance(
                            connection_manager.clone(),
                            topology_manager.clone(),
                            peer_discovery.clone(),
                            peers.clone(),
                            message_tx.clone(),
                        ).await;
                    }
                    _ = shutdown_signal.recv() => {
                        log::debug!("Shutting down maintenance task");
                        break;
                    }
                }
            }
        });
    }

    /// Perform network maintenance
    async fn perform_maintenance(
        _connection_manager: Arc<RwLock<ConnectionManager>>,
        _topology_manager: Arc<RwLock<TopologyManager>>,
        _peer_discovery: Arc<RwLock<PeerDiscovery>>,
        _peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        _message_tx: mpsc::Sender<NetworkMessage>,
    ) {
        // In a real implementation, this would:
        // 1. Ping peers to check connectivity
        // 2. Clean up stale connections
        // 3. Optimize network topology
        // 4. Request new peers if needed
    }

    /// Start message handler
    async fn start_message_handler(&self) {
        // We need a separate channel for message handling
        let message_tx = self._message_tx.clone();
        let mut message_rx = self.message_rx.lock().await.take().expect("Message receiver already taken");
        let connection_manager = self.connection_manager.clone();
        let broadcast_manager = self.broadcast_manager.clone();
        let peers = self.peers.clone();
        let response_tx = self.response_tx.clone();
        let mut shutdown_signal = self.shutdown_signal.subscribe();

        // Start message handler task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    maybe_msg = message_rx.recv() => {
                        match maybe_msg {
                            Some(msg) => {
                                Self::handle_network_message(
                                    msg,
                                    connection_manager.clone(),
                                    broadcast_manager.clone(),
                                    peers.clone(),
                                    response_tx.clone(),
                                ).await;
                            }
                            None => {
                                log::debug!("Network message channel closed");
                                break;
                            }
                        }
                    }
                    _ = shutdown_signal.recv() => {
                        log::debug!("Shutting down message handler");
                        break;
                    }
                }
            }
        });
    }

    /// Handle a network message
    async fn handle_network_message(
        msg: NetworkMessage,
        connection_manager: Arc<RwLock<ConnectionManager>>,
        broadcast_manager: Arc<RwLock<BroadcastManager>>,
        peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        response_tx: mpsc::Sender<NetworkResponse>,
    ) {
        match msg {
            NetworkMessage::Connect(addr) => {
                // Try to connect to peer
                let result = connection_manager.write().await.establish_outbound_connection(addr).await;
                let response = match result {
                    Ok(_) => NetworkResponse::ConnectionResult(Ok(())),
                    Err(e) => NetworkResponse::ConnectionResult(Err(e)),
                };

                if let Err(e) = response_tx.send(response).await {
                    log::warn!("Failed to send connection result: {}", e);
                }
            }
            NetworkMessage::Disconnect(peer_id, reason) => {
                // Disconnect from peer
                connection_manager.write().await.disconnect(peer_id, reason);
            }
            NetworkMessage::SendMessage(_peer_id, message) => {
                // Send message to specific peer
                // This would require accessing the actual connection
                let response = NetworkResponse::MessageSent(Ok(()));

                if let Err(e) = response_tx.send(response).await {
                    log::warn!("Failed to send message result: {}", e);
                }
            }
            NetworkMessage::Broadcast(protocol, message) => {
                // Broadcast message to peers
                let result = broadcast_manager.write().await.broadcast_message(protocol, message).await;
                let response = NetworkResponse::BroadcastResult(result);

                if let Err(e) = response_tx.send(response).await {
                    log::warn!("Failed to send broadcast result: {}", e);
                }
            }
            NetworkMessage::GetPeers => {
                // Get list of peers
                let peer_list: Vec<PeerInfo> = peers.read().await.values().cloned().collect();

                if let Err(e) = response_tx.send(NetworkResponse::PeerList(peer_list)).await {
                    log::warn!("Failed to send peer list: {}", e);
                }
            }
            NetworkMessage::PeerDiscovered(peer_info) => {
                // Store newly discovered peer
                peers.write().await.insert(peer_info.id, peer_info);
            }
            NetworkMessage::Shutdown => {
                // Signal shutdown
                if let Err(e) = response_tx.send(NetworkResponse::Shutdown).await {
                    log::warn!("Failed to send shutdown response: {}", e);
                }
            }
        }
    }

    /// Send a message through the network service
    pub async fn send_message(&self, peer_id: PeerId, message: Message) -> Result<(), String> {
        match self._message_tx.send(NetworkMessage::SendMessage(peer_id, message)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send message: {}", e)),
        }
    }

    /// Broadcast a message to the network
    pub async fn broadcast(&self, protocol: Protocol, message: Message) -> Result<(), String> {
        match self._message_tx.send(NetworkMessage::Broadcast(protocol, message)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to broadcast message: {}", e)),
        }
    }

    /// Get current peers
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>, String> {
        // This implementation has a bug - it creates a channel but doesn't register it correctly
        // We'll need to fix it with proper request/response handling
        let peers: Vec<PeerInfo> = self.peers.read().await.values().cloned().collect();
        Ok(peers)
    }

    /// Stop the network service
    pub async fn stop(&self) -> Result<(), String> {
        // Update status
        *self.status.write().await = NetworkStatus::Stopping;

        // Send shutdown signal
        let _ = self.shutdown_signal.send(());

        // Record a disconnection for security tracking
        for peer in self.peers.read().await.values() {
            self.record_disconnection(peer.address).await;
        }

        // Disconnect from all peers
        let peers: Vec<PeerId> = self.peers.read().await.keys().copied().collect();
        for peer_id in peers {
            if let Err(e) = self._message_tx.send(NetworkMessage::Disconnect(
                peer_id,
                DisconnectReason::Normal
            )).await {
                log::warn!("Failed to send disconnect message: {}", e);
            }
        }

        // Update status
        *self.status.write().await = NetworkStatus::Stopped;

        Ok(())
    }



    /// Clone implementation for NetworkService since we need this for NetworkServiceInterface
    pub fn clone(&self) -> Self {
        // Only clone what's needed for the service interface
        NetworkService {
            config: self.config.clone(),
            node: self.node.clone(),
            status: self.status.clone(),
            connection_manager: self.connection_manager.clone(),
            peer_discovery: self.peer_discovery.clone(),
            topology_manager: self.topology_manager.clone(),
            broadcast_manager: self.broadcast_manager.clone(),
            nat_traversal: self.nat_traversal.clone(),
            _message_tx: self._message_tx.clone(),
            message_rx: tokio::sync::Mutex::new(None),
            response_tx: self.response_tx.clone(),
            peers: self.peers.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
            metrics: self.metrics.clone(),
            block_propagator: self.block_propagator.clone(),
            peer_block_preferences: self.peer_block_preferences.clone(),
            protocol_negotiator: self.protocol_negotiator.clone(),
            sybil_protection: self.sybil_protection.clone(),
            dos_protection: self.dos_protection.clone(),
        }
    }

    // New methods for additional features

    /// Set up block propagation mechanisms
    async fn setup_block_propagation(&mut self) -> Result<(), String> {
        // Create a block propagator with compact blocks enabled
        let block_propagator = BlockPropagator::new(true, 0.8);
        self.block_propagator = Some(Arc::new(RwLock::new(block_propagator)));

        // Track peer preferences for block announcement formats
        let mut peer_preferences = HashMap::new();

        // For demonstration, set some preferences based on peer capabilities
        for (peer_id, peer_info) in self.peers.read().await.iter() {
            // In a real implementation, this would be determined by peer capabilities
            // For simplicity here, we'll use a basic heuristic
            if peer_info.protocol_version > 1 {
                peer_preferences.insert(*peer_id, BlockPreference::CompactBlocks);
            } else {
                peer_preferences.insert(*peer_id, BlockPreference::FullBlocks);
            }
        }

        *self.peer_block_preferences.write().await = peer_preferences;

        Ok(())
    }

    /// Propagate a block to connected peers
    pub async fn propagate_block(&self, block: Block) -> Result<(), String> {
        if let Some(propagator) = &self.block_propagator {
            let preferences = self.peer_block_preferences.read().await.clone();
            propagator.write().await.propagate_block(block, &preferences).await?;
        } else {
            return Err("Block propagator not initialized".to_string());
        }

        Ok(())
    }


    /// Negotiate protocol versions with a peer
    pub async fn negotiate_protocols(&self, peer_id: PeerId, remote_capabilities: ProtocolCapabilities) -> Result<(), String> {
        let negotiator = self.protocol_negotiator.read().await;
        let negotiated = negotiator.negotiate(&remote_capabilities);

        // Store the negotiated protocols for this peer
        self.store_peer_negotiated_protocols(peer_id, negotiated).await?;

        Ok(())
    }

    /// Store negotiated protocols for a peer
    async fn store_peer_negotiated_protocols(&self, peer_id: PeerId, negotiated: NegotiatedProtocols) -> Result<(), String> {
        // In a real implementation, this would store the protocols in a peer_protocols map
        // For this example, we'll just log the negotiated protocols
        log::info!("Negotiated protocols with peer {}: {:?}", peer_id, negotiated.protocols);
        log::info!("Negotiated features with peer {}: {:?}", peer_id, negotiated.features);

        Ok(())
    }

    /// Check if an incoming connection is allowed (avoid Sybil attacks)
    pub async fn is_connection_allowed(&self, addr: &SocketAddr) -> bool {
        self.sybil_protection.read().await.is_connection_allowed(addr)
    }

    /// Record a new connection for security tracking
    pub async fn record_connection(&self, addr: SocketAddr) {
        self.sybil_protection.write().await.record_connection(addr);
    }

    /// Record a disconnection for security tracking
    pub async fn record_disconnection(&self, addr: SocketAddr) {
        self.sybil_protection.write().await.record_disconnection(addr);
    }

    /// Check rate limits for a specific resource
    pub async fn check_rate_limit(&self, resource: ResourceType, peer_id: &PeerId, amount: u32) -> Result<(), String> {
        self.dos_protection.read().await.check_rate_limit(resource, peer_id, amount)
    }

    /// Handle a received compact block
    pub async fn handle_compact_block(&self, announcement: BlockAnnouncement) -> Result<Option<Block>, String> {
        if let Some(propagator) = &self.block_propagator {
            propagator.write().await.handle_compact_block(announcement).await
        } else {
            Err("Block propagator not initialized".to_string())
        }
    }

    /// Get current status
    pub async fn status(&self) -> NetworkStatus {
        self.status.read().await.clone()
    }

    /// Get connected peer count
    pub async fn connected_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}

// Implementación para NetworkServiceInterface
impl NetworkServiceInterfaceTrait for NetworkService {
    async fn connected_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    async fn connected_validator_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.values().filter(|info| {
            info.capabilities.contains(&"VALIDATOR".to_string())
        }).count()
    }

    async fn average_peer_latency(&self) -> Duration {
        // En una implementación real, esto calcularía la latencia media
        // Por ahora, devuelve un valor fijo
        Duration::from_millis(100)
    }

    async fn trigger_peer_discovery(&self) {
        // Realizar descubrimiento de pares
        if let Err(e) = self.bootstrap_peer_connections().await {
            log::warn!("Error al activar descubrimiento de pares: {}", e);
        }
    }

    async fn prioritize_validator_connections(&self) {
        // En una implementación real, esto buscaría y se conectaría a validadores
        log::info!("Priorizando conexiones a validadores");
    }
}

/// Implementación de LocalNetworkServiceInterface que usa NetworkService
pub struct LocalNetworkServiceInterface {
    service: Arc<NetworkService>,
}

impl LocalNetworkServiceInterface {
    pub fn new(service: Arc<NetworkService>) -> Self {
        Self { service }
    }
}

impl NetworkServiceInterfaceTrait for LocalNetworkServiceInterface {
    async fn connected_peer_count(&self) -> usize {
        self.service.connected_peer_count().await
    }

    async fn connected_validator_count(&self) -> usize {
        let peers = self.service.get_peers().await.unwrap_or_default();
        peers.iter().filter(|info| {
            info.capabilities.contains(&"VALIDATOR".to_string())
        }).count()
    }

    async fn average_peer_latency(&self) -> Duration {
        // En una implementación real, calcularía latencia real
        Duration::from_millis(100)
    }

    async fn trigger_peer_discovery(&self) {
        // En una implementación real, activaría descubrimiento de pares
        log::info!("Activando descubrimiento de pares por verificación de salud");
    }

    async fn prioritize_validator_connections(&self) {
        // En una implementación real, esto priorizaría conexiones a validadores
        log::info!("Priorizando conexiones con validadores por verificación de salud");
    }
}

// Default implementations for placeholder types
impl Default for PeerDiscovery {
    fn default() -> Self {
        // Create a minimal working implementation instead of panicking
        let config = crate::network::discovery::BootstrapConfig {
            bootstrap_nodes: Vec::new(),
            dns_seeds: Vec::new(),
            enable_local_discovery: false,
            static_peers: Vec::new(),
            dns_lookup_interval: Duration::from_secs(60),
        };
        PeerDiscovery::new(config)
    }
}

impl Default for TopologyManager {
    fn default() -> Self {
        // Create a minimal working implementation instead of panicking
        let config = crate::network::topology::TopologyConfig {
            target_outbound: 8,
            max_inbound: 125,
            max_peers_per_ip: 1,
            preferred_nodes: Vec::new(),
            preferred_regions: Vec::new(),
        };
        TopologyManager::new(config)
    }
}

// Removed duplicate implementation of Default for BroadcastManager since it is already defined in propagation.rs

impl Default for NatTraversal {
    fn default() -> Self {
        // Create a minimal working implementation instead of panicking
        NatTraversal::new(Vec::new(), false)
    }
}



