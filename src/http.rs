use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast::Sender as BroadcastSender, mpsc::UnboundedSender, watch::Sender as WatchSender};
use tracing::info;

use crate::config::{Config, ConfigRequest, Message};

pub struct HttpModule {
    name: &'static str,
    sender: BroadcastSender<Message>,
    shutdown: WatchSender<bool>,
    config_request: UnboundedSender<ConfigRequest>,
}

impl HttpModule {
    pub fn new(
        name: &'static str,
        sender: BroadcastSender<Message>,
        shutdown: WatchSender<bool>,
        config_request: UnboundedSender<ConfigRequest>,
    ) -> Self {
        Self {
            name,
            sender,
            shutdown,
            config_request,
        }
    }

    pub async fn run(self, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "http listening on");

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(pair) => pair,
                Err(err) => {
                    info!(error = ?err, "http shutting down");
                    return Err(err);
                }
            };
            let sender = self.sender.clone();
            let shutdown = self.shutdown.clone();
            let name = self.name;
            let config_request = self.config_request.clone();

            tokio::spawn(async move {
                let mut buffer = [0u8; 1024];
                let n = match socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let request = String::from_utf8_lossy(&buffer[..n]);
                let request_line = request.lines().next().unwrap_or_default();
                let request_body = request.split("\r\n\r\n").nth(1).unwrap_or("");
                let (status_line, body, content_type) = if request_line.starts_with("GET /send") {
                    let message = Message::Broadcast {
                        sender: name,
                        body: "hello from http".to_string(),
                    };
                    let _ = sender.send(message);
                    ("HTTP/1.1 200 OK\r\n", String::from("message sent\n"), "text/plain")
                } else if request_line.starts_with("GET /shutdown") {
                    let _ = shutdown.send(true);
                    ("HTTP/1.1 200 OK\r\n", String::from("shutting down\r\n"), "text/plain")
                } else if request_line.starts_with("GET /config") {
                    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                    let request = ConfigRequest::GetConfig {
                        requester: name,
                        response: response_tx,
                    };
                    let body = match config_request.send(request) {
                        Ok(_) => match response_rx.await {
                            Ok(config) => serde_json::to_string(&config).unwrap_or_else(|_| String::from("{}\n")),
                            Err(_) => String::from("{}\n"),
                        },
                        Err(_) => String::from("{}\n"),
                    };
                    ("HTTP/1.1 200 OK\r\n", body, "application/json")
                } else if request_line.starts_with("POST /set_config") {
                    match serde_json::from_str::<Config>(request_body) {
                        Ok(new_config) => {
                            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                            let request = ConfigRequest::SetConfig {
                                requester: name,
                                config: new_config,
                                response: response_tx,
                            };
                            let body = match config_request.send(request) {
                                Ok(_) => match response_rx.await {
                                    Ok(updated) => serde_json::to_string(&updated)
                                        .unwrap_or_else(|_| String::from("{}\n")),
                                    Err(_) => String::from("{}\n"),
                                },
                                Err(_) => String::from("{}\n"),
                            };
                            ("HTTP/1.1 200 OK\r\n", body, "application/json")
                        }
                        Err(_) => (
                            "HTTP/1.1 400 Bad Request\r\n",
                            String::from("invalid json\n"),
                            "text/plain",
                        ),
                    }
                } else {
                    info!(request_line = %request_line, "http request not found");
                    ("HTTP/1.1 404 Not Found\r\n", String::from("not found\n"), "text/plain")
                };

                let response = format!(
                    "{}content-length: {}\r\ncontent-type: {}\r\n\r\n{}",
                    status_line,
                    body.len(),
                    content_type,
                    body
                );

                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }
}

impl Drop for HttpModule {
    fn drop(&mut self) {
        info!("http dropping and shutting down");
    }
}
