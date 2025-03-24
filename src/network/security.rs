//! Network security enhancements

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::types::{DisconnectReason, PeerId};

/// IP subnet representation for grouping
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IpSubnet {
    V4(Ipv4Subnet),
    V6(Ipv6Subnet),
}

/// IPv4 subnet
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ipv4Subnet {
    pub base: [u8; 4],
    pub prefix_len: u8,
}

/// IPv6 subnet
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ipv6Subnet {
    pub base: [u8; 16],
    pub prefix_len: u8,
}

impl Ipv4Subnet {
    pub fn new(base: [u8; 4], prefix_len: u8) -> Self {
        Self { base, prefix_len }
    }
    
    pub fn contains(&self, ip: &Ipv4Addr) -> bool {
        let ip_octets = ip.octets();
        let full_bytes = self.prefix_len as usize / 8;
        
        // Check full bytes
        for i in 0..full_bytes {
            if self.base[i] != ip_octets[i] {
                return false;
            }
        }
        
        // Check partial byte if needed
        let remaining_bits = self.prefix_len % 8;
        if remaining_bits > 0 && full_bytes < 4 {
            let mask = !((1 << (8 - remaining_bits)) - 1);
            if (self.base[full_bytes] & mask) != (ip_octets[full_bytes] & mask) {
                return false;
            }
        }
        
        true
    }
}

impl Ipv6Subnet {
    pub fn new(base: Ipv6Addr, prefix_len: u8) -> Self {
        Self { base: base.octets(), prefix_len }
    }
}

/// Protection against Sybil attacks
pub struct SybilProtection {
    min_outbound_connections: usize,
    max_inbound_per_ip: usize,
    peer_reputation: ReputationTracker,
    ip_address_groups: HashMap<IpSubnet, usize>,
    address_restriction_level: AddressRestrictionLevel,
    connection_counts: HashMap<IpAddr, usize>,
    max_inbound_per_subnet: usize,
    asn_connection_counts: HashMap<u32, usize>,
    ip_to_asn_lookup: IpAsnLookup,
    max_inbound_per_asn: usize,
}

/// ASN lookup service
pub struct IpAsnLookup {
    // In a real implementation, this would have a database
    // For now, just placeholder methods
}

impl IpAsnLookup {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn get_asn_for_ip(&self, _ip: IpAddr) -> Option<u32> {
        // In a real implementation, this would perform a lookup
        Some(64496) // Example private ASN
    }
}

/// Reputation tracking for peers
pub struct ReputationTracker {
    scores: HashMap<PeerId, f32>,
    min_acceptable_score: f32,
}

impl ReputationTracker {
    pub fn new(min_score: f32) -> Self {
        Self {
            scores: HashMap::new(),
            min_acceptable_score: min_score,
        }
    }
    
    pub fn get_score(&self, peer_id: &PeerId) -> f32 {
        *self.scores.get(peer_id).unwrap_or(&0.5) // Default neutral score
    }
    
    pub fn update_score(&mut self, peer_id: PeerId, adjustment: f32) {
        let current = self.get_score(&peer_id);
        let new_score = (current + adjustment).clamp(0.0, 1.0);
        self.scores.insert(peer_id, new_score);
    }
    
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        self.get_score(peer_id) < self.min_acceptable_score
    }
}

/// Levels of IP address restriction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressRestrictionLevel {
    None,
    LimitPerIp,
    LimitPerSubnet,
    LimitPerAsn,
}

impl SybilProtection {
    /// Create a new Sybil protection system
    pub fn new(
        min_outbound: usize,
        max_per_ip: usize,
        max_per_subnet: usize,
        max_per_asn: usize,
        restriction_level: AddressRestrictionLevel,
    ) -> Self {
        Self {
            min_outbound_connections: min_outbound,
            max_inbound_per_ip: max_per_ip,
            peer_reputation: ReputationTracker::new(0.2),
            ip_address_groups: HashMap::new(),
            address_restriction_level: restriction_level,
            connection_counts: HashMap::new(),
            max_inbound_per_subnet: max_per_subnet,
            asn_connection_counts: HashMap::new(),
            ip_to_asn_lookup: IpAsnLookup::new(),
            max_inbound_per_asn: max_per_asn,
        }
    }

    /// Check if a connection should be allowed
    pub fn is_connection_allowed(&self, addr: &SocketAddr) -> bool {
        // Check IP-based connection limits
        let ip = addr.ip();
        
        match self.address_restriction_level {
            AddressRestrictionLevel::None => true,
            
            AddressRestrictionLevel::LimitPerIp => {
                // Check if we've reached max connections for this IP
                let current = self.connection_counts.get(&ip).unwrap_or(&0);
                *current < self.max_inbound_per_ip
            },
            
            AddressRestrictionLevel::LimitPerSubnet => {
                // Check subnet restrictions
                let subnet = self.get_subnet_for_ip(ip);
                let current = self.ip_address_groups.get(&subnet).unwrap_or(&0);
                *current < self.max_inbound_per_subnet
            },
            
            AddressRestrictionLevel::LimitPerAsn => {
                // Check ASN restrictions (most restrictive)
                if let Some(asn) = self.ip_to_asn_lookup.get_asn_for_ip(ip) {
                    let current = self.asn_connection_counts.get(&asn).unwrap_or(&0);
                    *current < self.max_inbound_per_asn
                } else {
                    // Fall back to subnet restriction if ASN lookup fails
                    let subnet = self.get_subnet_for_ip(ip);
                    let current = self.ip_address_groups.get(&subnet).unwrap_or(&0);
                    *current < self.max_inbound_per_subnet
                }
            }
        }
    }
    
    /// Get the subnet for an IP address
    fn get_subnet_for_ip(&self, ip: IpAddr) -> IpSubnet {
        match ip {
            IpAddr::V4(ipv4) => {
                let prefix = ipv4.octets();
                // Group by /24 subnet for IPv4
                IpSubnet::V4(Ipv4Subnet::new([prefix[0], prefix[1], prefix[2], 0], 24))
            },
            IpAddr::V6(ipv6) => {
                // Group by /48 subnet for IPv6
                IpSubnet::V6(Ipv6Subnet::new(ipv6, 48))
            }
        }
    }
    
    /// Record a new connection
    pub fn record_connection(&mut self, addr: SocketAddr) {
        let ip = addr.ip();
        
        // Track connection count for this IP
        *self.connection_counts.entry(ip).or_insert(0) += 1;
        
        // Track subnet groups
        let subnet = self.get_subnet_for_ip(ip);
        *self.ip_address_groups.entry(subnet).or_insert(0) += 1;
        
        // Track ASN if available
        if let Some(asn) = self.ip_to_asn_lookup.get_asn_for_ip(ip) {
            *self.asn_connection_counts.entry(asn).or_insert(0) += 1;
        }
    }
    
    /// Record a disconnection
    pub fn record_disconnection(&mut self, addr: SocketAddr) {
        let ip = addr.ip();
        
        // Update connection count for this IP
        if let Some(count) = self.connection_counts.get_mut(&ip) {
            *count = count.saturating_sub(1);
        }
        
        // Update subnet groups
        let subnet = self.get_subnet_for_ip(ip);
        if let Some(count) = self.ip_address_groups.get_mut(&subnet) {
            *count = count.saturating_sub(1);
        }
        
        // Update ASN if available
        if let Some(asn) = self.ip_to_asn_lookup.get_asn_for_ip(ip) {
            if let Some(count) = self.asn_connection_counts.get_mut(&asn) {
                *count = count.saturating_sub(1);
            }
        }
    }
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    capacity: u32,
    refill_rate: u32, // Units per second
    current_tokens: AtomicU32,
    last_refill: AtomicU64, // Timestamp as u64
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            refill_rate,
            current_tokens: AtomicU32::new(capacity),
            last_refill: AtomicU64::new(unix_timestamp_ms()),
        }
    }
    
    /// Try to consume tokens from the bucket
    pub fn try_consume(&self, amount: u32) -> bool {
        // Refill tokens based on time elapsed
        self.refill_if_needed();
        
        // Try to consume tokens
        let mut current = self.current_tokens.load(Ordering::Relaxed);
        loop {
            if current < amount {
                return false; // Not enough tokens
            }
            
            let new_value = current - amount;
            match self.current_tokens.compare_exchange(
                current, 
                new_value,
                Ordering::SeqCst,
                Ordering::Relaxed
            ) {
                Ok(_) => return true, // Successfully consumed tokens
                Err(actual) => current = actual, // Someone else changed the value, retry
            }
        }
    }
    
    /// Refill tokens based on elapsed time
    fn refill_if_needed(&self) {
        let now = unix_timestamp_ms();
        let last = self.last_refill.load(Ordering::Relaxed);
        
        // Calculate how many tokens to add based on elapsed time
        let elapsed_seconds = ((now - last) as f64) / 1000.0;
        if elapsed_seconds < 0.01 {
            return; // Too soon to refill
        }
        
        // Calculate tokens to add
        let new_tokens = (elapsed_seconds * self.refill_rate as f64) as u32;
        if new_tokens == 0 {
            return;
        }
        
        // Add tokens up to capacity
        let mut current = self.current_tokens.load(Ordering::Relaxed);
        loop {
            let new_value = (current + new_tokens).min(self.capacity);
            match self.current_tokens.compare_exchange(
                current, 
                new_value,
                Ordering::SeqCst,
                Ordering::Relaxed
            ) {
                Ok(_) => {
                    // Update last refill time
                    self.last_refill.store(now, Ordering::Relaxed);
                    break;
                }
                Err(actual) => current = actual, // Someone else changed the value, retry
            }
        }
    }
}

/// DoS protection system
pub struct DoSProtection {
    rate_limiters: HashMap<ResourceType, RateLimiter>,
    ban_threshold: u32,
    penalty_scores: HashMap<PeerId, u32>,
    recent_violators: ExpiringCache<PeerId, ViolationType>,
    whitelist: HashSet<IpAddr>,
    peer_to_addr: HashMap<PeerId, IpAddr>,
    banned_addresses: ExpiringSet<IpAddr>,
    connection_manager: ConnectionManagerInterface,
}

/// Connection manager interface
pub struct ConnectionManagerInterface {
    // En una implementación real, esto tendría una referencia a ConnectionManager
    disconnect_tx: Option<tokio::sync::mpsc::Sender<(PeerId, DisconnectReason)>>,
}

impl ConnectionManagerInterface {
    pub fn new() -> Self {
        Self {
            disconnect_tx: None,
        }
    }
    
    pub fn with_sender(disconnect_tx: tokio::sync::mpsc::Sender<(PeerId, DisconnectReason)>) -> Self {
        Self {
            disconnect_tx: Some(disconnect_tx),
        }
    }
    
    pub async fn disconnect(&self, peer_id: PeerId, reason: DisconnectReason) {
        if let Some(tx) = &self.disconnect_tx {
            // Enviar solicitud de desconexión
            if let Err(e) = tx.send((peer_id, reason)).await {
                log::error!("Error al enviar solicitud de desconexión: {}", e);
            }
        } else {
            // Si no hay sender, sólo loguear
            log::info!("Se desconectaría el peer {} con razón {:?}", peer_id, reason);
        }
    }
}

/// Cache with expiring entries
pub struct ExpiringCache<K, V> {
    entries: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> ExpiringCache<K, V> {
    /// Create a new expiring cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }
    
    /// Insert a key-value pair with expiration
    pub fn insert(&mut self, key: K, value: V, ttl: Duration) {
        self.entries.insert(key, (value, Instant::now() + ttl));
    }
    
    /// Get a value if it exists and hasn't expired
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let is_expired = {
            if let Some((_, expiry)) = self.entries.get(key) {
                Instant::now() >= *expiry
            } else {
                false
            }
        };
        if is_expired {
            self.entries.remove(key);
            None
        } else {
            self.entries.get(key).map(|(value, _)| value)
        }
    }
    
    /// Clean up expired entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, (_, expiry)| *expiry > now);
    }
}

/// Resource types for rate limiting
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceType {
    InboundConnections,
    RpcRequests,
    BlockRequests,
    TransactionAnnouncements,
    TotalBandwidth,
}

/// Types of violations
#[derive(Clone, Copy, Debug)]
pub enum ViolationType {
    RateLimit(ResourceType),
    ProtocolViolation,
    InvalidMessage,
    InvalidBlock,
    InvalidTransaction,
}

impl ViolationType {
    /// Get penalty points for this violation type
    pub fn penalty_points(&self) -> u32 {
        match self {
            ViolationType::RateLimit(_) => 1, // Minor
            ViolationType::ProtocolViolation => 5, // Moderate
            ViolationType::InvalidMessage => 2, // Minor
            ViolationType::InvalidBlock => 10, // Severe
            ViolationType::InvalidTransaction => 3, // Moderate
        }
    }
}

impl DoSProtection {
    /// Create a new DoS protection system
    pub fn new(connection_manager: ConnectionManagerInterface, whitelist: HashSet<IpAddr>) -> Self {
        let mut rate_limiters = HashMap::new();
        
        // Initialize rate limiters for different resource types
        rate_limiters.insert(ResourceType::InboundConnections, RateLimiter::new(5, 1)); // 5 per sec
        rate_limiters.insert(ResourceType::BlockRequests, RateLimiter::new(10, 2)); // 10 per 2 sec
        rate_limiters.insert(ResourceType::TransactionAnnouncements, RateLimiter::new(100, 50)); // 100 per 50 sec
        rate_limiters.insert(ResourceType::TotalBandwidth, RateLimiter::new(1_000_000, 500_000)); // 1MB/500ms
        
        Self {
            rate_limiters,
            ban_threshold: 20,
            penalty_scores: HashMap::new(),
            recent_violators: ExpiringCache::new(Duration::from_secs(900)), // 15 min history
            whitelist,
            peer_to_addr: HashMap::new(),
            banned_addresses: ExpiringSet::new(Duration::from_secs(86400)), // 24h ban
            connection_manager,
        }
    }

    /// Check if an action would exceed rate limits
    pub fn check_rate_limit(&self, resource: ResourceType, peer_id: &PeerId, amount: u32) -> Result<(), String> {
        // Check if peer is whitelisted
        if let Some(addr) = self.peer_to_addr.get(peer_id) {
            if self.whitelist.contains(addr) {
                return Ok(());
            }
        }
        
        // Get appropriate rate limiter
        let limiter = match self.rate_limiters.get(&resource) {
            Some(l) => l,
            None => return Err("Rate limiter not found".to_string()),
        };
            
        // Try to consume tokens
        if !limiter.try_consume(amount) {
            return Err("Rate limit exceeded".to_string());
        }
        
        Ok(())
    }
    
    /// Record a violation by a peer
    pub fn record_violation(&mut self, peer_id: PeerId, violation: ViolationType) {
        // Record the violation
        self.recent_violators.insert(peer_id, violation, Duration::from_secs(300));
        
        // Update penalty score
        let score = self.penalty_scores.entry(peer_id).or_insert(0);
        *score += violation.penalty_points();
        
        // Check if ban threshold reached
        if *score >= self.ban_threshold {
            self.ban_peer(peer_id);
        }
    }
    
    /// Ban a peer
    fn ban_peer(&mut self, peer_id: PeerId) {
        // Get peer's address
        if let Some(addr) = self.peer_to_addr.get(&peer_id) {
            // Add to ban list with expiry
            let ban_duration = self.calculate_ban_duration(&peer_id);
            self.banned_addresses.insert(*addr, ban_duration);
            
            // Disconnect peer
            self.connection_manager.disconnect(peer_id, DisconnectReason::PeerBanned);
            
            log::warn!("Banned peer {} for {} seconds due to DoS violations", 
                     peer_id, ban_duration.as_secs());
        }
    }
    
    /// Calculate ban duration based on violation history
    fn calculate_ban_duration(&self, peer_id: &PeerId) -> Duration {
        let score = self.penalty_scores.get(peer_id).unwrap_or(&0);
        
        // Exponential backoff based on score
        let hours = (2_u64.pow(*score as u32 / 5) - 1).min(24 * 7); // Max 1 week
        Duration::from_secs(hours * 3600)
    }
}

/// Set with entries that expire after a time
pub struct ExpiringSet<T> {
    entries: HashMap<T, Instant>,
    ttl: Duration,
}

impl<T: std::hash::Hash + Eq + Clone> ExpiringSet<T> {
    /// Create a new expiring set with the given TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }
    
    /// Insert an item with custom expiration
    pub fn insert(&mut self, item: T, ttl: Duration) {
        self.entries.insert(item, Instant::now() + ttl);
    }
    
    /// Check if an item is in the set and not expired
    pub fn contains(&mut self, item: &T) -> bool {
        if let Some(expiry) = self.entries.get(item) {
            if *expiry > Instant::now() {
                return true;
            } else {
                // Remove expired entry
                self.entries.remove(item);
            }
        }
        false
    }
    
    // Insert with default TTL
    pub fn insert_with_default_ttl(&mut self, item: T) {
        self.entries.insert(item, Instant::now() + self.ttl);
    }
}

/// Get current UNIX timestamp in milliseconds
fn unix_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}
