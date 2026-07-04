use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Broadcast {
        sender: &'static str,
        body: String,
    },
    Ping {
        sender: &'static str,
    },
    Pong {
        sender: &'static str,
    },
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
        let ping = Message::Ping {
            sender: "http",
        };
        let pong = Message::Pong {
            sender: "config",
        };

        assert!(matches!(
            ping,
            Message::Ping {
                sender: "http",
            }
        ));
        assert!(matches!(
            pong,
            Message::Pong {
                sender: "config",
            }
        ));
    }
}
