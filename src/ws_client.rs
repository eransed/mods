use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{info, error};

pub struct WsClient {
    url: String,
}

impl WsClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn run(self) {
        info!(url = %self.url, "ws_client connecting...");

        let (mut socket, _response) = match connect_async(&self.url).await {
            Ok(pair) => pair,
            Err(err) => {
                error!(error = ?err, "ws_client failed to connect");
                return;
            }
        };

        info!("ws_client connected to websocket server");

        while let Some(message_result) = socket.next().await {
            match message_result {
                Ok(WsMessage::Frame(frame)) => {
                    info!(frame = %frame, "ws_client received frame");
                }
                Ok(WsMessage::Text(text)) => {
                    info!(text = %text, "ws_client received text");
                }
                Ok(WsMessage::Binary(data)) => {
                    info!(bytes = ?data, "ws_client received binary");
                }
                Ok(WsMessage::Ping(payload)) => {
                    info!(payload = ?payload, "ws_client received ping");
                }
                Ok(WsMessage::Pong(payload)) => {
                    info!(payload = ?payload, "ws_client received pong");
                }
                Ok(WsMessage::Close(frame)) => {
                    info!(frame = ?frame, "ws_client websocket closed");
                    break;
                }
                Err(err) => {
                    error!(error = ?err, "ws_client websocket error");
                    break;
                }
            }
        }

        info!("ws_client shutting down");
    }
}

impl Drop for WsClient {
    fn drop(&mut self) {
        info!("ws_client dropping and shutting down");
    }
}
