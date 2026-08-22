use futures_util::StreamExt;
use tokio::sync::watch;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info, trace};
use types::Config;

pub struct WsClient {
  url: String,
}

impl WsClient {
  pub fn new(url: String) -> Self {
    Self { url }
  }

  pub async fn run(
    self,
    mut config_rx: watch::Receiver<Config>,
    mut shutdown_rx: watch::Receiver<bool>,
  ) {
    let mut url = self.url.clone();
    loop {
      info!(url = %url, "ws_client connecting...");

      let (mut socket, _response) = match connect_async(&url).await {
        Ok(pair) => pair,
        Err(err) => {
          error!(error = ?err, "ws_client failed to connect");
          tokio::select! {
            _ = shutdown_rx.changed() => return,
            result = config_rx.changed() => {
              if result.is_err() { return; }
              url = format!("ws://127.0.0.1:{}", config_rx.borrow().ws_port);
              continue;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => continue,
          }
        }
      };

      info!("ws_client connected to websocket server");

      loop {
        tokio::select! {
            _ = shutdown_rx.changed() => return,
            result = config_rx.changed() => {
              if result.is_err() { return; }
              url = format!("ws://127.0.0.1:{}", config_rx.borrow().ws_port);
              break;
            }
            message_result = socket.next() => {
              let Some(message_result) = message_result else { break; };
              match message_result {
          Ok(WsMessage::Frame(frame)) => {
            debug!(frame = %frame, "ws_client received frame");
          }
          Ok(WsMessage::Text(text)) => {
            if text.len() > 300 {
              trace!("ws_client received text, {} bytes", text.len());
            } else {
              trace!("ws_client received text, {} bytes: {}", text.len(), text);
            }
          }
          Ok(WsMessage::Binary(data)) => {
            debug!(bytes = ?data, "ws_client received binary");
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
          }
      }
    }
  }
}

impl Drop for WsClient {
  fn drop(&mut self) {
    info!("ws_client dropping and shutting down");
  }
}
