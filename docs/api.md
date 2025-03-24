# ProzChain Network Layer API Documentation

This document provides an overview of the ProzChain Network Layer API, focusing on how to interact with the network layer from other components.

## Primary Interfaces

### NetworkService

The `NetworkService` is the main entry point for interaction with the network layer. It provides methods for:

- **Starting/stopping the network service**
- **Sending messages to specific peers**
- **Broadcasting messages to the network**
- **Managing peer connections**

```rust
// Create a network service
let (service, responses) = NetworkService::new(config).await?;

// Start the service
service.start().await?;

// Send a message to a specific peer
let message = Message::new(Protocol::BlockSync, 0x01, payload);
service.send_message(peer_id, message).await?;

// Broadcast a message to the network
service.broadcast(Protocol::TransactionPropagation, message).await?;

// Get list of connected peers
let peers = service.get_peers().await?;

// Gracefully shut down the service
service.stop().await?;
```

### Message Handling

To handle incoming messages, you need to register message handlers for specific protocols:

```rust
// Implement the MessageHandler trait
struct MyMessageHandler;

impl MessageHandler for MyMessageHandler {
    fn handle_message(&self, message: Message, context: &MessageContext) -> Result<(), String> {
        // Process the message
        println!("Received message: {:?}", message);
        Ok(())
    }

    fn supported_protocol(&self) -> Protocol {
        Protocol::BlockSync
    }

    fn supported_message_types(&self) -> Vec<u16> {
        vec![0x01, 0x02]  // Handle message types 1 and 2
    }
}

// Register with the network service
service.register_message_handler(Box::new(MyMessageHandler));
```

## Error Handling

Most methods return a `Result` type that should be properly handled. Common errors include:

- Connection failures
- Timeout errors
- Protocol violations
- Rate limiting errors

Example:

```rust
match service.send_message(peer_id, message).await {
    Ok(_) => println!("Message sent successfully"),
    Err(e) => match e {
        NetworkError::ConnectionTimeout => {
            println!("Connection timed out, peer may be offline")
        },
        NetworkError::RateLimitExceeded => {
            println!("Rate limit exceeded, try again later")
        },
        _ => println!("Error sending message: {}", e),
    }
}
```

## Event Handling

The `NetworkService` also provides an event stream for asynchronous notifications:

```rust
// Process network responses
tokio::spawn(async move {
    while let Some(response) = responses.recv().await {
        match response {
            NetworkResponse::PeerList(peers) => {
                println!("Received peer list with {} peers", peers.len());
            },
            NetworkResponse::MessageSent(result) => {
                println!("Message sent result: {:?}", result);
            },
            NetworkResponse::BroadcastResult(result) => {
                println!("Broadcast result: {:?}", result);
            },
            NetworkResponse::Shutdown => {
                println!("Network service has shut down");
                break;
            },
            // Handle other response types
        }
    }
});
```

## Configuration

The network layer is configured through the `NetworkConfig` struct:

```rust
let config = NetworkConfig {
    node_config: node_config,
    listen_addresses: vec!["0.0.0.0:30333".to_string()],
    bootstrap_nodes: vec!["bootstrap1.prozchain.io:30333".to_string()],
    dns_seeds: vec!["seed1.prozchain.io".to_string()],
    max_peers: 25,
    connection_timeout: Duration::from_secs(10),
    ping_interval: Duration::from_secs(60),
    peer_exchange_interval: Duration::from_secs(300),
    enable_upnp: true,
    enable_nat_traversal: true,
    stun_servers: vec!["stun.prozchain.io:3478".to_string()],
    whitelist: None,
    blacklist: None,
};
```

For more detailed information, see the internal API documentation.
