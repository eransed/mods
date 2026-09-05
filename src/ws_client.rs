use futures_util::StreamExt;
use tokio::sync::watch;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info};
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
              // Reconnect to the websocket server at the configured port.
              url = format!("ws://127.0.0.1:{}", config_rx.borrow().general_config.ws_port.value);
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
            // Reconnect after a websocket port configuration change.
            url = format!("ws://127.0.0.1:{}", config_rx.borrow().general_config.ws_port.value);
            break;
          }
          message_result = socket.next() => {
            let Some(message_result) = message_result else { break; };
            match message_result {
              Ok(WsMessage::Frame(frame)) => {
                debug!(frame = %frame, "received frame");
              }
              Ok(WsMessage::Text(text)) => {
                if text.len() > 300 {
                  // debug!("received text, {} bytes", text.len());
                } else {
                  // debug!("received text, {} bytes: {}", text.len(), text);
                }
              }
              Ok(WsMessage::Binary(data)) => {
                debug!(bytes = ?data, "received binary");
              }
              Ok(WsMessage::Ping(payload)) => {
                info!(payload = ?payload, "received ping");
              }
              Ok(WsMessage::Pong(payload)) => {
                info!(payload = ?payload, "received pong");
              }
              Ok(WsMessage::Close(frame)) => {
                info!(frame = ?frame, "websocket closed");
                break;
              }
              Err(err) => {
                error!(error = ?err, "websocket error");
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
    info!("dropping and shutting down");
  }
}
