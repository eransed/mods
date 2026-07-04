use futures_util::{SinkExt, StreamExt};
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
use tracing::{error, info};

use crate::config::Message;

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

        tokio::spawn(async move {
            while let Ok(Message::Broadcast { sender, body }) = receiver.recv().await {
                let text = format!("{sender}: {body}");
                info!(%sender, %text, "ws_server broadcasting internal message");
                let mut clients = clients.lock().await;
                clients.retain(|client| client.send(WsMessage::Text(text.clone())).is_ok());
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

                info!("ws_server websocket client connected");

                while let Some(message_result) = read.next().await {
                    match message_result {
                        Ok(WsMessage::Text(text)) => {
                            let broadcast_message = Message::Broadcast {
                                sender: name,
                                body: text.clone(),
                            };
                            let _ = sender.send(broadcast_message);
                            info!(text = %text, "ws_server received websocket text");
                        }
                        Ok(WsMessage::Close(_)) => {
                            info!("ws_server websocket client disconnected");
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            error!(error = ?err, "ws_server websocket error");
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
        info!("ws_server dropping and shutting down");
    }
}
