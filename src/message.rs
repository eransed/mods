use serde::{Deserialize, Serialize};

use crate::udp_discovery_server::DiscoveryEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
  Broadcast { sender: &'static str, body: String },
  Ping { sender: &'static str },
  Pong { sender: &'static str },
  Discovery(DiscoveryEvent),
  SystemStatus { cpu_percent: f32, ram_percent: f32, pid_mem_bytes: u64 },
  OpenProtocolState(OpenProtocolState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenProtocolState {
  pub name: String,
  pub ip: String,
  pub port: u16,
  pub connected: bool,
  pub ping_ms: Option<u64>,
  pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicMessage {
  pub topic: String,
}

#[cfg(test)]
mod tests {
  use super::Message;

  #[test]
  fn ping_and_pong_can_hold_timestamps() {
    let ping = Message::Ping { sender: "http" };
    let pong = Message::Pong { sender: "config" };

    assert!(matches!(ping, Message::Ping { sender: "http" }));
    assert!(matches!(pong, Message::Pong { sender: "config" }));
  }
}
