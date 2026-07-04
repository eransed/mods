use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Broadcast {
        sender: &'static str,
        body: String,
    },
    Ping {
        sender: &'static str,
        timestamp: u64,
    },
    Pong {
        sender: &'static str,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicMessage {
    pub topic: String,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn ping_and_pong_can_hold_timestamps() {
        let ping = Message::Ping {
            sender: "http",
            timestamp: 42,
        };
        let pong = Message::Pong {
            sender: "config",
            timestamp: 84,
        };

        assert!(matches!(
            ping,
            Message::Ping {
                sender: "http",
                timestamp: 42
            }
        ));
        assert!(matches!(
            pong,
            Message::Pong {
                sender: "config",
                timestamp: 84
            }
        ));
    }
}
