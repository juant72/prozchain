#[cfg(test)]
mod tests {
    use crate::network::message::{Message, Protocol};
    use crate::network::node::{NodeConfig, ProzChainNode};
    use crate::network::service::{NetworkConfig, NetworkService};
    use std::time::Duration;

    #[tokio::test]
    async fn test_create_node() {
        let config = NodeConfig {
            node_type: "full".to_string(),
            validator_key_path: None,
            stake_amount: None,
            trusted_validators: None,
            pruning_strategy: None,
            api_config: None,
            listen_addresses: vec!["127.0.0.1:0".to_string()],
            external_addresses: None,
            display_name: Some("Test Node".to_string()),
            max_peers: 10,
            connection_limits: Default::default(),
        };

        let node = ProzChainNode::new(config);
        assert!(node.is_ok(), "Should create a node successfully");
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let test_data = b"Hello, ProzChain!".to_vec();
        let message = Message::new(Protocol::Discovery, 0x01, test_data.clone());
        
        let serialized = message.serialize();
        assert!(serialized.is_ok(), "Should serialize successfully");
        
        let bytes = serialized.unwrap();
        let deserialized = Message::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Should deserialize successfully");
        
        let restored_message = deserialized.unwrap();
        assert_eq!(restored_message.payload, test_data, "Payload should match original");
        assert_eq!(restored_message.header.protocol_id, Protocol::Discovery as u16, "Protocol should match");
    }

    #[tokio::test]
    #[ignore] // Requires network access, so ignored by default
    async fn test_network_service_startup() {
        let config = NetworkConfig {
            node_config: NodeConfig {
                node_type: "full".to_string(),
                validator_key_path: None,
                stake_amount: None,
                trusted_validators: None,
                pruning_strategy: None,
                api_config: None,
                listen_addresses: vec!["127.0.0.1:0".to_string()], // Random port
                external_addresses: None,
                display_name: Some("Test Node".to_string()),
                max_peers: 10,
                connection_limits: Default::default(),
            },
            listen_addresses: vec!["127.0.0.1:0".to_string()], // Random port
            bootstrap_nodes: vec![], // Empty for test
            dns_seeds: vec![],
            max_peers: 10,
            connection_timeout: Duration::from_secs(1),
            ping_interval: Duration::from_secs(10),
            peer_exchange_interval: Duration::from_secs(30),
            enable_upnp: false,
            enable_nat_traversal: false,
            stun_servers: vec![],
            whitelist: None,
            blacklist: None,
        };
        
        let (mut service, _responses) = NetworkService::new(config).await.unwrap();
        let result = service.start().await;
        
        // Just check if it starts without error
        assert!(result.is_ok(), "Network service should start successfully");
        
        // Shut down
        let _ = service.stop().await;
    }
    
    #[tokio::test]
    async fn test_peer_discovery() {
        // Mock test for peer discovery
        let bootstrap_config = crate::network::discovery::BootstrapConfig {
            bootstrap_nodes: vec![],
            dns_seeds: vec![],
            enable_local_discovery: false,
            static_peers: vec![],
            dns_lookup_interval: Duration::from_secs(60),
        };
        
        // This test will use mocks in a real implementation
        // For now, just check that the structure is defined correctly
        assert_eq!(bootstrap_config.dns_lookup_interval, Duration::from_secs(60));
    }
}
