use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tokio::sync::{
    broadcast::Sender as BroadcastSender, mpsc::UnboundedSender, watch::Sender as WatchSender,
};
use tracing::info;

use crate::{
    config::{Config, ConfigRequest},
    message::Message,
};

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
        let sender = self.sender.clone();
        let mut receiver = sender.subscribe();
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let Message::Ping {
                    sender: origin,
                    timestamp,
                } = message
                {
                    let _ = sender.send(Message::Pong {
                        sender: self.name,
                        timestamp,
                    });
                    let _ = origin;
                }
            }
        });

        let state = HttpState {
            name: self.name,
            sender: self.sender.clone(),
            shutdown: self.shutdown.clone(),
            config_request: self.config_request.clone(),
        };

        let app = Router::new()
            .route("/send", get(send_handler))
            .route("/shutdown", get(shutdown_handler))
            .route("/config", get(config_handler))
            .route("/ping", get(ping_handler))
            .route("/reset_config", get(reset_config_handler))
            .route("/set_config", post(set_config_handler))
            .with_state(state);

        info!(%addr, "http listening on");
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service())
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

#[derive(Clone)]
struct HttpState {
    name: &'static str,
    sender: BroadcastSender<Message>,
    shutdown: WatchSender<bool>,
    config_request: UnboundedSender<ConfigRequest>,
}

async fn send_handler(State(state): State<HttpState>) -> impl IntoResponse {
    let message = Message::Broadcast {
        sender: state.name,
        body: "hello from http".to_string(),
    };
    let _ = state.sender.send(message);
    (StatusCode::OK, "message sent\n")
}

async fn shutdown_handler(State(state): State<HttpState>) -> impl IntoResponse {
    let _ = state.shutdown.send(true);
    (StatusCode::OK, "shutting down\r\n")
}

async fn config_handler(State(state): State<HttpState>) -> impl IntoResponse {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let request = ConfigRequest::GetConfig {
        requester: state.name,
        response: response_tx,
    };

    match state.config_request.send(request) {
        Ok(_) => match response_rx.await {
            Ok(config) => (StatusCode::OK, Json(config)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config response failed").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config request failed").into_response(),
    }
}

async fn set_config_handler(
    State(state): State<HttpState>,
    Json(new_config): Json<Config>,
) -> impl IntoResponse {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let request = ConfigRequest::SetConfig {
        requester: state.name,
        config: new_config,
        response: response_tx,
    };

    match state.config_request.send(request) {
        Ok(_) => match response_rx.await {
            Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config response failed").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config request failed").into_response(),
    }
}

async fn reset_config_handler(State(state): State<HttpState>) -> impl IntoResponse {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let request = ConfigRequest::ResetConfig {
        requester: state.name,
        response: response_tx,
    };

    match state.config_request.send(request) {
        Ok(_) => match response_rx.await {
            Ok(default_config) => (StatusCode::OK, Json(default_config)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config response failed").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config request failed").into_response(),
    }
}

async fn ping_handler(State(state): State<HttpState>) -> impl IntoResponse {
    let started_at = Instant::now();
    let timestamp = started_at.elapsed().as_micros() as u64;
    let _ = state.sender.send(Message::Ping {
        sender: state.name,
        timestamp,
    });

    let mut latencies = HashMap::new();
    let mut receiver = state.sender.subscribe();
    let mut total_latency = 0u64;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(200);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, receiver.recv()).await {
            Ok(Ok(Message::Pong {
                sender,
                timestamp: pong_timestamp,
            })) => {
                let latency_us = started_at.elapsed().as_micros() as u64 - pong_timestamp;
                latencies.insert(sender.to_string() + "_us", latency_us);
                total_latency = total_latency.saturating_add(latency_us);
            }
            Ok(Ok(_)) => {}
            Ok(Err(_)) | Err(_) => break,
        }
    }

    let response = serde_json::json!({
        "latencies": latencies,
        "total_latency_us": total_latency,
    });

    (StatusCode::OK, Json(response)).into_response()
}

impl Drop for HttpModule {
    fn drop(&mut self) {
        info!("http dropping and shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::{net::SocketAddr, sync::Arc, time::Duration};
    use tokio::{net::TcpListener, sync::Mutex};

    async fn spawn_http_module() -> (SocketAddr, tokio::task::JoinHandle<std::io::Result<()>>) {
        let (sender, _) = tokio::sync::broadcast::channel(16);
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        let (config_request_tx, mut config_request_rx) = tokio::sync::mpsc::unbounded_channel();
        let current_config = Arc::new(Mutex::new(Config::default()));

        tokio::spawn({
            let current_config = current_config.clone();
            async move {
                while let Some(request) = config_request_rx.recv().await {
                    match request {
                        ConfigRequest::GetConfig { response, .. } => {
                            let _ = response.send(current_config.lock().await.clone());
                        }
                        ConfigRequest::SetConfig {
                            config, response, ..
                        } => {
                            let response_config = config.clone();
                            *current_config.lock().await = config;
                            let _ = response.send(response_config);
                        }
                        ConfigRequest::ResetConfig { response, .. } => {
                            let default = Config::default();
                            *current_config.lock().await = default.clone();
                            let _ = response.send(default);
                        }
                    }
                }
            }
        });

        let module = HttpModule::new("http", sender, shutdown_tx, config_request_tx);
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();
        let actual_addr = listener.local_addr().unwrap();
        drop(listener);

        let handle = tokio::spawn(async move { module.run(actual_addr).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        (actual_addr, handle)
    }

    #[tokio::test]
    async fn send_endpoint_returns_ok() {
        let (addr, handle) = spawn_http_module().await;
        let response = reqwest::get(format!("http://{addr}/send")).await.unwrap();

        assert!(response.status().is_success());
        let body = response.text().await.unwrap();
        assert_eq!(body, "message sent\n");

        handle.abort();
    }

    #[tokio::test]
    async fn config_endpoint_returns_default_config() {
        let (addr, handle) = spawn_http_module().await;
        let response = reqwest::get(format!("http://{addr}/config")).await.unwrap();

        assert!(response.status().is_success());
        let config: Config = response.json().await.unwrap();
        assert_eq!(config, Config::default());

        handle.abort();
    }

    #[tokio::test]
    async fn set_config_endpoint_returns_updated_config() {
        let (addr, handle) = spawn_http_module().await;
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{addr}/set_config"))
            .json(&Config {
                http_port: 8080,
                ws_port: 8085,
            })
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let config: Config = response.json().await.unwrap();
        assert_eq!(
            config,
            Config {
                http_port: 8080,
                ws_port: 8085
            }
        );

        handle.abort();
    }

    #[tokio::test]
    async fn reset_config_endpoint_returns_default_config() {
        let (addr, handle) = spawn_http_module().await;
        let response = reqwest::get(format!("http://{addr}/reset_config"))
            .await
            .unwrap();

        assert!(response.status().is_success());
        let config: Config = response.json().await.unwrap();
        assert_eq!(config, Config::default());

        handle.abort();
    }

    #[tokio::test]
    async fn ping_endpoint_returns_json_latency_map() {
        let (addr, handle) = spawn_http_module().await;
        let response = reqwest::get(format!("http://{addr}/ping")).await.unwrap();

        assert!(response.status().is_success());
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.is_object());

        handle.abort();
    }
}
