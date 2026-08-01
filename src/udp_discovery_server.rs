use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::message::Message;

// ============================================================================
// Phase 1: Core Data Structures (Foundation)
// ============================================================================

/// DiscoveryMessage — serialized/deserialized over UDP multicast packets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
  pub node_name: String,
  pub ip_address: IpAddr,
  pub http_port: u16,
  pub system_type: String,
}

/// PeerInfo — local representation of a discovered peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInfo {
  pub node_name: String,
  pub ip_address: IpAddr,
  pub http_port: u16,
  pub system_type: String,
  /// Timestamp (seconds since UNIX_EPOCH) when peer was last seen
  pub last_seen: u64,
}

impl PeerInfo {
  /// Create a new PeerInfo with current timestamp
  fn new(node_name: String, ip_address: IpAddr, http_port: u16, system_type: String) -> Self {
    let last_seen = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    Self { node_name, ip_address, http_port, system_type, last_seen }
  }

  /// Update last_seen timestamp
  fn update_timestamp(&mut self) {
    self.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
  }

  /// Create PeerInfo from DiscoveryMessage
  fn from_discovery_message(msg: DiscoveryMessage) -> Self {
    Self::new(msg.node_name, msg.ip_address, msg.http_port, msg.system_type)
  }
}

/// DiscoveryEvent — events broadcast to all modules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryEvent {
  PeerDiscovered(PeerInfo),
  PeerUpdated(PeerInfo),
}

/// DiscoveryCommand — commands from other modules to discovery
pub enum DiscoveryCommand {
  GetPeers(oneshot::Sender<Vec<PeerInfo>>),
  SendDiscoveryBeacon(Option<oneshot::Sender<Result<(), String>>>),
  ClearPeers,
}

// ============================================================================
// Phase 2: UDP Socket & Network I/O (Network Layer)
// ============================================================================

/// DiscoveryServer — main UDP multicast discovery server
pub struct DiscoveryServer {
  /// Thread-safe peer registry (keyed by node_name)
  peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
  /// Async UDP socket bound and joined to multicast group
  socket: tokio::net::UdpSocket,
  /// Local peer identity
  local_peer_info: PeerInfo,
  /// Broadcast sender for notifying other modules
  broadcast_sender: broadcast::Sender<Message>,
  /// Receiver for discovery commands
  command_rx: mpsc::UnboundedReceiver<DiscoveryCommand>,
}

impl Drop for DiscoveryServer {
  fn drop(&mut self) {
    info!("discovery server dropping and shutting down");
  }
}

impl DiscoveryServer {
  const MULTICAST_ADDR: &'static str = "239.1.1.1:8125";
  const BIND_ADDR: &'static str = "0.0.0.0:8125";
  const BUFFER_SIZE: usize = 1024;

  /// Create a new DiscoveryServer
  ///
  /// # Arguments
  /// * `local_node_name` - This node's identifier
  /// * `local_ip` - This node's IP address
  /// * `local_http_port` - HTTP port for this node
  /// * `system_type` - System/device type identifier
  /// * `broadcast_sender` - Reference to the central message broadcast hub
  /// * `command_rx` - Channel for receiving discovery commands
  ///
  /// # Returns
  /// Result with the initialized DiscoveryServer or an error string
  pub async fn new(
    local_node_name: String,
    local_ip: IpAddr,
    local_http_port: u16,
    system_type: String,
    broadcast_sender: broadcast::Sender<Message>,
    command_rx: mpsc::UnboundedReceiver<DiscoveryCommand>,
  ) -> Result<Self, String> {
    // Setup socket
    let socket = Self::setup_socket().await?;

    let local_peer_info = PeerInfo::new(local_node_name, local_ip, local_http_port, system_type);

    Ok(Self {
      peers: Arc::new(RwLock::new(HashMap::new())),
      socket,
      local_peer_info,
      broadcast_sender,
      command_rx,
    })
  }

  /// Setup UDP socket: bind to BIND_ADDR and join multicast group
  async fn setup_socket() -> Result<tokio::net::UdpSocket, String> {
    // Create and bind standard UDP socket first
    info!("Binding to {}", Self::BIND_ADDR);
    let std_socket =
      UdpSocket::bind(Self::BIND_ADDR).map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

    // Use socket2 to set socket options
    let socket2_obj = socket2::Socket::from(std_socket);
    socket2_obj
      .set_reuse_address(true)
      .map_err(|e| format!("Failed to set SO_REUSEADDR: {}", e))?;

    // Platform-specific SO_REUSEPORT configuration
    // #[cfg(not(target_os = "windows"))]
    // {
    //     socket2_obj
    //         .set_reuse_port(true)
    //         .map_err(|e| format!("Failed to set SO_REUSEPORT: {}", e))?;
    // }

    // Convert back to std::net::UdpSocket
    let std_socket = std::net::UdpSocket::from(socket2_obj);

    // Join multicast group on all available interfaces (0.0.0.0)
    let multicast_addr = "239.1.1.1"
      .parse::<std::net::IpAddr>()
      .map_err(|e| format!("Failed to parse multicast address: {}", e))?;

    std_socket
      .join_multicast_v4(
        &multicast_addr
          .to_string()
          .parse()
          .map_err(|e| format!("Multicast address parse error: {}", e))?,
        &"0.0.0.0".parse().map_err(|e| format!("0.0.0.0 parse error: {}", e))?,
      )
      .map_err(|e| format!("Failed to join multicast group: {}", e))?;

    // Set non-blocking and wrap in tokio UdpSocket
    std_socket.set_nonblocking(true).map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    let tokio_socket = tokio::net::UdpSocket::from_std(std_socket)
      .map_err(|e| format!("Failed to wrap in tokio UdpSocket: {}", e))?;

    info!("UDP multicast socket setup complete: {}", Self::MULTICAST_ADDR);
    Ok(tokio_socket)
  }

  /// Handle an incoming discovery packet
  async fn handle_discovery_packet(&self, msg: DiscoveryMessage) {
    // Ignore packets from self
    if msg.node_name == self.local_peer_info.node_name {
      debug!("Ignoring discovery packet from self ({})", msg.node_name);
      return;
    }

    let peer_info = PeerInfo::from_discovery_message(msg.clone());
    let mut peers = self.peers.write().await;

    match peers.get_mut(&peer_info.node_name) {
      Some(existing_peer) => {
        // Peer already known; check if info changed
        let info_changed = existing_peer.ip_address != peer_info.ip_address
          || existing_peer.http_port != peer_info.http_port
          || existing_peer.system_type != peer_info.system_type;

        existing_peer.update_timestamp();

        if info_changed {
          *existing_peer = peer_info.clone();
          debug!("Peer updated: {:?}", peer_info.node_name);
          // Broadcast PeerUpdated event
          let event = DiscoveryEvent::PeerUpdated(peer_info);
          let _ = self.broadcast_sender.send(Message::Discovery(event)).map_err(|_| ());
        } else {
          debug!("Peer info unchanged: {}", peer_info.node_name);
        }
      }
      None => {
        // New peer discovered
        debug!("New peer discovered: {:?}", peer_info);
        peers.insert(peer_info.node_name.clone(), peer_info.clone());
        // Broadcast PeerDiscovered event
        let event = DiscoveryEvent::PeerDiscovered(peer_info);
        let _ = self.broadcast_sender.send(Message::Discovery(event)).map_err(|_| ());
      }
    }
  }

  /// Send a discovery beacon (multicast packet with local peer info)
  async fn send_discovery_packet(&self) -> Result<(), String> {
    let discovery_msg = DiscoveryMessage {
      node_name: self.local_peer_info.node_name.clone(),
      ip_address: self.local_peer_info.ip_address,
      http_port: self.local_peer_info.http_port,
      system_type: self.local_peer_info.system_type.clone(),
    };

    let serialized = bincode::serialize(&discovery_msg)
      .map_err(|e| format!("Failed to serialize discovery message: {}", e))?;

    let multicast_addr = Self::MULTICAST_ADDR
      .parse::<SocketAddr>()
      .map_err(|e| format!("Failed to parse multicast address: {}", e))?;

    self
      .socket
      .send_to(&serialized, multicast_addr)
      .await
      .map_err(|e| format!("Failed to send discovery packet: {}", e))?;

    debug!("Sent discovery beacon for {}", self.local_peer_info.node_name);
    Ok(())
  }

  // ========================================================================
  // Phase 3: Command Handling & API (Module Interface)
  // ========================================================================

  /// Main async event loop — select between socket receive and command handler
  pub async fn run(mut self) {
    info!("Starting UDP Discovery Server");

    let mut buf = vec![0u8; Self::BUFFER_SIZE];

    loop {
      tokio::select! {
          // Handle incoming UDP packets
          result = self.socket.recv_from(&mut buf) => {
              match result {
                  Ok((n, _addr)) => {
                      match bincode::deserialize::<DiscoveryMessage>(&buf[..n]) {
                          Ok(msg) => {
                              self.handle_discovery_packet(msg).await;
                          }
                          Err(e) => {
                              warn!("Failed to deserialize discovery packet: {}", e);
                          }
                      }
                  }
                  Err(e) => {
                      error!("Error receiving from UDP socket: {}", e);
                  }
              }
          }

          // Handle discovery commands
          Some(command) = self.command_rx.recv() => {
              self.handle_command(command).await;
          }
      }
    }
  }

  /// Handle discovery commands
  async fn handle_command(&self, command: DiscoveryCommand) {
    match command {
      DiscoveryCommand::GetPeers(tx) => {
        let peers = self.peers.read().await;
        let peer_list: Vec<PeerInfo> = peers.values().cloned().collect();
        let _ = tx.send(peer_list);
        info!("GetPeers: returned {} peers", peers.len());
      }
      DiscoveryCommand::SendDiscoveryBeacon(tx) => {
        info!("Sending discovery beacon");
        let result = self.send_discovery_packet().await;
        if let Some(response_tx) = tx {
          let _ = response_tx.send(result);
        }
      }
      DiscoveryCommand::ClearPeers => {
        let mut peers = self.peers.write().await;
        peers.clear();
        info!("Cleared all discovered peers");
      }
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_peer_info_creation() {
    let peer = PeerInfo::new(
      "node1".to_string(),
      "192.168.1.100".parse().unwrap(),
      8080,
      "Router".to_string(),
    );

    assert_eq!(peer.node_name, "node1");
    assert_eq!(peer.http_port, 8080);
    assert_eq!(peer.system_type, "Router");
    assert!(peer.last_seen > 0);
  }

  #[test]
  fn test_discovery_message_serialization() {
    let msg = DiscoveryMessage {
      node_name: "node1".to_string(),
      ip_address: "192.168.1.100".parse().unwrap(),
      http_port: 8080,
      system_type: "Gateway".to_string(),
    };

    let serialized = bincode::serialize(&msg).expect("Serialization failed");
    let deserialized: DiscoveryMessage =
      bincode::deserialize(&serialized).expect("Deserialization failed");

    assert_eq!(deserialized.node_name, msg.node_name);
    assert_eq!(deserialized.ip_address, msg.ip_address);
    assert_eq!(deserialized.http_port, msg.http_port);
    assert_eq!(deserialized.system_type, msg.system_type);
  }

  #[test]
  fn test_peer_info_from_discovery_message() {
    let msg = DiscoveryMessage {
      node_name: "node2".to_string(),
      ip_address: "192.168.1.101".parse().unwrap(),
      http_port: 9000,
      system_type: "Device".to_string(),
    };

    let peer = PeerInfo::from_discovery_message(msg);
    assert_eq!(peer.node_name, "node2");
    assert_eq!(peer.http_port, 9000);
  }
}
