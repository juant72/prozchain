# Network Layer Documentation

## 1. Overview
The network layer builds a robust P2P infrastructure, enabling low-latency connections and high resilience against attacks (e.g., DoS, eclipse). Key concepts are defined for clarity.

## 2. Architecture

### 2.1 Protocol Stack
Each layer in the stack handles a specific function:
```
+-----------------------+
| Application Protocol  |
+-----------------------+
| Message Protocol      |
+-----------------------+
| Transport Protocol    |
+-----------------------+
| Discovery Protocol    |
```
*Explanation:* This modular separation allows updating individual components without affecting the whole.

### 2.2 Key Components
- **Discovery Service:** Locates and connects new peers.
- **Connection Manager:** Establishes and maintains connections.
- **Message Router:** Directs messages to their destination.
- **Reputation System:** Evaluates peer reliability.
- **NAT Traversal:** Enables connectivity through firewalls.
- **Rate Limiter:** Prevents DoS attacks.

## 3. Peer Discovery

### 3.1 Discovery Protocol
A modified DHT (based on Kademlia) is used to find nodes.
```rust
struct DiscoveryConfig {
    bootstrap_nodes: Vec<Multiaddr>,
    target_connections: usize,
    max_connections: usize,
    discovery_interval: Duration,
    node_id: PeerId,
    local_key: Keypair,
}
```
*Explanation:*  
- Bootstrap nodes initiate the connection.
- The DHT enables decentralized peer discovery.

### 3.2 Bootstrap Process
Steps:
1. Connect to predefined bootstrap nodes.
2. Retrieve peer lists.
3. Begin periodic discovery cycles.
4. Persist discovered peers.

### 3.3 Peer Storage and Selection
Uses persistent storage and scoring to ensure diverse and reliable peers.

## 4. Transport Protocol

### 4.1 Multiple Transport Options
Supports QUIC, TCP+Noise, and WebRTC to dynamically select the best connection.
```rust
enum TransportType { Quic, TcpNoise, WebRTC }

struct TransportManager {
    supported_transports: Vec<TransportType>,
    active_connections: HashMap<PeerId, TransportConnection>,
    connection_limits: ConnectionLimits,
}
```
*Explanation:* Dynamic transport selection improves latency and security.

### 4.2 Connection Management
Implements pooling, circuit breaking, and exponential backoff.
```rust
async fn establish_connection(peer_id: PeerId, addrs: Vec<Multiaddr>) -> Result<Connection> {
    // ...existing code...
    // Attempts connections in parallel, negotiates protocols, and sets up an encrypted channel.
}
```

## 5. Message Protocol

### 5.1 Message Types
Defines types for transactions, blocks, state, consensus, etc.
```rust
enum MessageType {
    Transaction, Block, BlockRequest, BlockResponse,
    StateRequest, StateResponse, Consensus, Attestation,
    Gossip, Ping, Pong,
}
```
*Explanation:* Classifying messages facilitates prioritization and duplicate filtering.

### 5.2 Message Handling Pipeline
Handles deserialization, validation, routing, and response.
```rust
struct MessageProcessor {
    handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
    validation_rules: HashMap<MessageType, Box<dyn ValidationRule>>,
    metrics: MessageMetrics,
}

impl MessageProcessor {
    pub async fn process_message(&self, msg: NetworkMessage) -> Result<()> {
        if !self.validate_message(&msg) { return Err(Error::ValidationFailed); }
        if let Some(handler) = self.handlers.get(&msg.message_type) { 
            handler.handle_message(msg).await?;
        }
        Ok(())
    }
    fn validate_message(&self, msg: &NetworkMessage) -> bool {
        // ...existing code...
    }
}
```
*Explanation:* Ensures messages are valid and processed safely.

### 5.3 Message Prioritization
Priority levels:
- **Critical:** Consensus messages.
- **High:** New blocks and attestations.
- **Medium:** Transactions and requests.
- **Low:** Peer discovery and state synchronization.

## 6. Gossip Protocol

### 6.1 Gossipsub Implementation
Gossipsub is used to efficiently disseminate messages.
```rust
struct GossipConfig { 
    topic_score_params: HashMap<TopicId, ScoreParams>,
    mesh_outbound_min: usize,
    mesh_outbound_max: usize,
    gossip_factor: f64,
    history_length: usize,
    history_gossip: usize,
}
```
*Explanation:* Topic scoring helps avoid duplicates and ensures a robust mesh.

### 6.2 Topic Structure
Defines topics for blocks, transactions, attestations, etc.

### 6.3 Message Deduplication
Uses Bloom filters and time-based expiration to prevent redundant message forwarding.

## 7. Security Measures

### 7.1 Sybil Attack Protection
Implements a rate limiter and reputation checks.
```rust
struct RateLimiter { 
    global_limits: RateLimits,
    per_peer_limits: HashMap<PeerId, RateLimits>,
    ip_limits: HashMap<IpAddr, RateLimits>,
    time_window: Duration,
}
```
*Explanation:* Prevents flooding and malicious behavior.

### 7.2 Eclipse Attack Prevention
Rotation and diversity in peer selection reduce this risk.

### 7.3 Transport Layer Security
All traffic is encrypted using protocols ensuring Perfect Forward Secrecy (PFS).

## 8. Implementation Details
Utilizes libraries such as `libp2p`, `quinn`, `tokio`, and `protobuf`.

## 9. Optimizations
Message compression, dynamic bandwidth management, and connection pooling.

## 10. Metrics and Monitoring
Integration with Prometheus and tracing dashboards for real‑time monitoring.

## 11. Testing Infrastructure
Simulated environments, chaos testing, and DoS simulations.

## 12. References
Documentation and specifications for Kademlia, QUIC, and libp2p.
