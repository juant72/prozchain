# ProzChain Network Layer

A high-performance blockchain network layer implemented in Rust, focusing on security, efficiency, and scalability.

## Features

- **P2P Architecture**: Optimized peer-to-peer networking with different node types and roles
- **Peer Discovery**: Multiple discovery mechanisms including bootstrap nodes, DNS seeds, and peer exchange
- **NAT Traversal**: Techniques for connecting nodes behind firewalls and NAT devices
- **Connection Management**: Efficient handling of peer connections with health monitoring
- **Message Protocols**: Well-defined message formats and versioning for network communication
- **Message Propagation**: Optimized broadcast and gossip protocols for efficient information spread
- **Network Security**: Protection against Sybil attacks, Eclipse attacks, and DDoS
- **Network Monitoring**: Comprehensive metrics and health checking

## Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Cargo

### Building the Project

```bash
# Clone the repository
git clone https://github.com/prozchain/prozchain.git
cd prozchain

# Build the project
cargo build --release
```

### Running a Node

```bash
# Run with default configuration
./target/release/prozchain

# Run with custom configuration
./target/release/prozchain --config my_config.toml
```

## Configuration

ProzChain uses TOML for configuration. A default configuration file is provided at `config/default.toml`.

Key configuration options:

- `node.type`: Type of node to run ("validator", "full", "light", "archive", "rpc")
- `network.listen_addresses`: Addresses to listen for incoming connections
- `network.bootstrap_nodes`: Initial nodes to connect to
- `network.max_peers`: Maximum number of peer connections to maintain

## Architecture

The network layer is composed of several interconnected components:

1. **Node Service**: Central coordination of all network activities
2. **Connection Manager**: Establishes and maintains peer connections
3. **Peer Discovery**: Finds and connects to peers in the network
4. **Message Handler**: Processes incoming and outgoing messages
5. **Broadcast Manager**: Efficiently propagates messages through the network
6. **Security Components**: Provides protection against various network attacks

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

ProzChain builds upon research and implementations from various blockchain projects and networking protocols.
