//! ProzChain - A high-performance blockchain built in Rust
//!
//! This is the main entry point for the ProzChain node application.

use prozchain_lib::network::node::NodeConfig;
use prozchain_lib::network::service::{NetworkConfig, NetworkService, NetworkResponse};
use prozchain_lib::init;
use log::{info, warn, error};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar la biblioteca
    init()?;
    
    // Analizar argumentos de línea de comandos
    let args: Vec<String> = env::args().collect();
    let config_path = parse_config_path(&args).unwrap_or_else(|| {
        info!("Usando configuración por defecto");
        PathBuf::from("config/default.toml")
    });
    
    // Cargar configuración
    info!("Cargando configuración desde {:?}", config_path);
    let config = load_config(&config_path)?;
    
    // Crear y iniciar el nodo
    info!("Inicializando nodo ProzChain...");
    let (network_service, mut network_responses) = NetworkService::new(config).await
        .map_err(|e| format!("Error al crear el servicio de red: {}", e))?;
    
    // Procesar respuestas de la red en una tarea en segundo plano
    tokio::spawn(async move {
        while let Some(response) = network_responses.recv().await {
            match response {
                NetworkResponse::PeerList(peers) => {
                    info!("Descubiertos {} peers", peers.len());
                },
                NetworkResponse::ConnectionResult(result) => {
                    match result {
                        Ok(_) => info!("Conexión establecida satisfactoriamente"),
                        Err(e) => warn!("Conexión fallida: {}", e),
                    }
                },
                NetworkResponse::MessageSent(result) => {
                    if let Err(e) = result {
                        warn!("Error al enviar mensaje: {}", e);
                    }
                },
                NetworkResponse::BroadcastResult(result) => {
                    if let Err(e) = result {
                        warn!("Error al difundir mensaje: {}", e);
                    }
                },
                NetworkResponse::Shutdown => {
                    info!("El servicio de red se ha apagado");
                    break;
                },
            }
        }
    });
    
    // Iniciar el servicio de red
    info!("Iniciando servicio de red...");
    network_service.start().await
        .map_err(|e| format!("Error al iniciar el servicio de red: {}", e))?;
    
    // Mostrar información del nodo
    let peer_count = network_service.connected_peer_count().await;
    let status = network_service.status().await;
    info!("Nodo ProzChain ejecutándose con estado {:?} y {} peers", status, peer_count);
    
    // Esperar señal CTRL+C
    info!("Presiona CTRL+C para detener el nodo");
    tokio::signal::ctrl_c().await?;
    
    // Detener el servicio de red
    info!("Deteniendo nodo ProzChain...");
    network_service.stop().await
        .map_err(|e| format!("Error durante el apagado: {}", e))?;
    
    info!("Nodo detenido correctamente");
    
    Ok(())
}

/// Analizar argumentos para encontrar la ruta de configuración
fn parse_config_path(args: &[String]) -> Option<PathBuf> {
    for i in 0..args.len() - 1 {
        if args[i] == "--config" || args[i] == "-c" {
            return Some(PathBuf::from(&args[i+1]));
        }
    }
    None
}

/// Cargar configuración desde archivo
fn load_config(path: &PathBuf) -> Result<NetworkConfig, Box<dyn std::error::Error>> {
    // Intentar parsear la configuración TOML
    let config_contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            warn!("No se pudo leer el archivo de configuración: {}", e);
            // Devolver configuración por defecto si no se puede leer el archivo
            return Ok(default_network_config());
        }
    };
    
    match toml::from_str::<config::Config>(&config_contents) {
        Ok(toml_config) => {
            // Convertir configuración TOML a nuestra NetworkConfig
            Ok(convert_toml_to_network_config(toml_config))
        },
        Err(e) => {
            error!("Error al parsear el archivo de configuración: {}", e);
            // Devolver configuración por defecto si falla el parseo
            Ok(default_network_config())
        }
    }
}

/// Configuración de red por defecto
fn default_network_config() -> NetworkConfig {
    NetworkConfig {
        node_config: NodeConfig {
            node_type: "full".to_string(),
            validator_key_path: None,
            stake_amount: None,
            trusted_validators: None,
            pruning_strategy: None,
            api_config: None,
            listen_addresses: vec!["0.0.0.0:30333".to_string()],
            external_addresses: None,
            display_name: Some("ProzChain Test Node".to_string()),
            max_peers: 25,
            connection_limits: Default::default(),
        },
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
        stun_servers: vec![
            "stun.prozchain.io:3478".to_string(),
        ],
        whitelist: None,
        blacklist: None,
    }
}

/// Convertir configuración TOML a NetworkConfig
fn convert_toml_to_network_config(toml_config: config::Config) -> NetworkConfig {
    // Valores por defecto para los límites de conexión
    let connection_limits = prozchain_lib::network::node::ConnectionLimits {
        max_inbound: toml_config.network.limits.max_inbound,
        target_outbound: toml_config.network.limits.target_outbound,
        max_peers_per_ip: toml_config.network.limits.max_peers_per_ip,
    };
    
    // Convertir configuración del nodo
    let node_config = NodeConfig {
        node_type: toml_config.node.type_,
        validator_key_path: None, // Se parsearía del TOML en una implementación completa
        stake_amount: None, // Se parsearía del TOML en una implementación completa
        trusted_validators: None, // Se parsearía del TOML en una implementación completa
        pruning_strategy: None, // Se parsearía del TOML en una implementación completa
        api_config: None, // Se parsearía del TOML en una implementación completa
        listen_addresses: toml_config.network.listen_addresses.clone(),
        external_addresses: Some(toml_config.network.external_addresses.clone()),
        display_name: Some(toml_config.node.display_name),
        max_peers: toml_config.node.max_peers,
        connection_limits,
    };
    
    // Crear configuración de red usando todos los campos requeridos y completando campos faltantes con valores por defecto
    NetworkConfig {
        node_config,
        listen_addresses: toml_config.network.listen_addresses.clone(),
        bootstrap_nodes: toml_config.network.bootstrap_nodes.clone(),
        dns_seeds: toml_config.network.dns_seeds.clone(),
        max_peers: toml_config.node.max_peers,
        connection_timeout: Duration::from_secs(toml_config.network.connection_timeout_seconds),
        ping_interval: Duration::from_secs(toml_config.network.ping_interval_seconds),
        peer_exchange_interval: Duration::from_secs(toml_config.network.peer_exchange_interval_seconds),
        enable_upnp: toml_config.network.enable_upnp,
        enable_nat_traversal: toml_config.network.enable_nat_traversal,
        stun_servers: toml_config.network.stun_servers.clone(),
        whitelist: None, // Se parsearía del TOML en una implementación completa
        blacklist: None, // Se parsearía del TOML en una implementación completa
    }
}

/// Estructuras de configuración TOML
mod config {
    use serde::Deserialize;
    
    #[derive(Deserialize)]
    pub struct Config {
        pub node: Node,
        pub network: Network,
        pub log: Option<Log>,
    }
    
    #[derive(Deserialize)]
    pub struct Node {
        #[serde(rename = "type")]
        pub type_: String,
        pub display_name: String,
        pub max_peers: usize,
    }
    
    #[derive(Deserialize)]
    pub struct Network {
        pub listen_addresses: Vec<String>,
        #[serde(default)]
        pub external_addresses: Vec<String>,
        pub bootstrap_nodes: Vec<String>,
        pub dns_seeds: Vec<String>,
        pub connection_timeout_seconds: u64,
        pub ping_interval_seconds: u64,
        pub peer_exchange_interval_seconds: u64,
        pub enable_upnp: bool,
        pub enable_nat_traversal: bool,
        pub stun_servers: Vec<String>,
        pub limits: NetworkLimits,
    }
    
    #[derive(Deserialize)]
    pub struct NetworkLimits {
        pub max_inbound: usize,
        pub target_outbound: usize,
        pub max_peers_per_ip: usize,
    }
    
    #[derive(Deserialize)]
    pub struct Log {
        pub level: String,
        pub enable_file_logging: bool,
        pub log_file: String,
    }
}
