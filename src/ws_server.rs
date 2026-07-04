use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{
    Mutex,
    broadcast::Sender,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info, warn};

use crate::message::{Message, TopicMessage};

#[derive(Debug, Deserialize)]
struct IncomingTopicMessage {
    topic: String,
}

fn parse_incoming_message(text: &str) -> Option<Message> {
    serde_json::from_str::<IncomingTopicMessage>(text)
        .ok()
        .and_then(|message| match message.topic.as_str() {
            "ping" => Some(Message::Ping {
                sender: "ws_client",
            }),
            "pong" => Some(Message::Pong {
                sender: "ws_client",
            }),
            _ => None,
        })
}

fn encode_topic_message(message: &Message) -> Option<String> {
    match message {
        Message::Pong { sender } => serde_json::to_string(&TopicMessage {
            topic: "pong".to_string(),
        })
        .ok()
        .map(|json| {
            let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
            value["sender"] = serde_json::Value::String(sender.to_string());
            value.to_string()
        }),
        _ => None,
    }
}

pub struct WsServer {
    name: &'static str,
    sender: Sender<Message>,
    clients: Arc<Mutex<Vec<UnboundedSender<WsMessage>>>>,
}

impl WsServer {
    pub fn new(name: &'static str, sender: Sender<Message>) -> Self {
        Self {
            name,
            sender,
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(self, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "ws_server websocket server listening on");

        let clients = self.clients.clone();
        let mut receiver = self.sender.subscribe();
        let broadcast_sender = self.sender.clone();

        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                match message {
                    Message::Broadcast { sender, body } => {
                        let text = format!("{sender}: {body}");
                        debug!(%sender, %text, "ws_server broadcasting internal message");
                        let mut clients = clients.lock().await;
                        clients.retain(|client| client.send(WsMessage::Text(text.clone())).is_ok());
                    }
                    Message::Pong { sender } => {
                        if let Some(text) =
                            encode_topic_message(&Message::Pong { sender })
                        {
                            let mut clients = clients.lock().await;
                            clients.retain(|client| {
                                client.send(WsMessage::Text(text.clone())).is_ok()
                            });
                        }
                    }
                    Message::Ping { .. } => {
                        // sleep for 450 ms:
                        // tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = broadcast_sender.send(Message::Pong {
                            sender: "ws_server",
                        });
                    }
                }
            }
        });

        while let Ok((stream, _peer)) = listener.accept().await {
            let clients = self.clients.clone();
            let sender = self.sender.clone();
            let name = self.name;

            tokio::spawn(async move {
                let websocket = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(err) => {
                        error!(error = ?err, "ws_server websocket handshake failed");
                        return;
                    }
                };

                let client_addr = websocket.get_ref().peer_addr().unwrap();
                let local_addr = websocket.get_ref().local_addr().unwrap();
                let (mut write, mut read) = websocket.split();
                let (tx, mut rx): (UnboundedSender<WsMessage>, UnboundedReceiver<WsMessage>) =
                    unbounded_channel();
                clients.lock().await.push(tx);

                let write_task = tokio::spawn(async move {
                    while let Some(message) = rx.recv().await {
                        if write.send(message).await.is_err() {
                            break;
                        }
                    }
                });

                info!(
                    "ws_server websocket client connected: {}, local: {}",
                    client_addr, local_addr
                );

                while let Some(message_result) = read.next().await {
                    match message_result {
                        Ok(WsMessage::Text(text)) => {
                            if let Some(message) = parse_incoming_message(&text) {
                                let _ = sender.send(message);
                                debug!(text = %text, "ws_server received websocket topic message: {}", client_addr);
                            } else {
                                let broadcast_message = Message::Broadcast {
                                    sender: name,
                                    body: text.clone(),
                                };
                                let _ = sender.send(broadcast_message);
                                debug!(text = %text, "ws_server received websocket text: {}", client_addr);
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            warn!("ws_server websocket client disconnected {}", client_addr);
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            error!(error = ?err, "ws_server websocket error {}", client_addr);
                            break;
                        }
                    }
                }

                let _ = write_task.await;
                let mut clients = clients.lock().await;
                clients.retain(|client| !client.is_closed());
            });
        }

        info!("ws_server shutting down");
        Ok(())
    }
}

impl Drop for WsServer {
    fn drop(&mut self) {
        debug!("ws_server dropping and shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_topic_message, parse_incoming_message};
    use crate::message::Message;

    #[test]
    fn parses_ping_payloads_from_json() {
        let message = parse_incoming_message(r#"{"topic":"ping"}"#);
        assert_eq!(
            message,
            Some(Message::Ping {
                sender: "ws_client",
            })
        );
    }

    #[test]
    fn encodes_pong_payloads_for_clients() {
        let encoded = encode_topic_message(&Message::Pong {
            sender: "ws_server",
        });

        let value: serde_json::Value = serde_json::from_str(encoded.as_deref().unwrap()).unwrap();
        assert_eq!(value["topic"], "pong");
        assert_eq!(value["sender"], "ws_server");
    }
}
