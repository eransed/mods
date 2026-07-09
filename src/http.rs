use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, Query, State},
    http::{
        Method, Request, StatusCode,
        header::{
            ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_METHOD, HeaderValue,
        },
    },
    middleware::{self, Next},
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
use tracing::{debug, info};

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
                if let Message::Ping { sender: origin } = message {
                    let _ = sender.send(Message::Pong { sender: self.name });
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
            .layer(middleware::from_fn(log_request))
            .with_state(state);

        info!(%addr, "http listening on");
        let listener = TcpListener::bind(addr).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
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

async fn log_request(
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let is_preflight = request
        .headers()
        .contains_key(ACCESS_CONTROL_REQUEST_METHOD)
        && request.method() == Method::OPTIONS;
    let query = request.uri().query().unwrap_or_default().to_string();
    info!(
        peer_addr = %peer_addr,
        path = %request.uri().path(),
        query = %query,
        "req"
    );

    let mut response = if is_preflight {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };

    let headers = response.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type, Authorization"),
    );

    response
}

fn parse_max_response_time_micros(query_params: &HashMap<String, String>) -> u64 {
    query_params
        .get("max_response_time_micros")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(500_000)
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

async fn ping_handler(
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<HashMap<String, String>>,
    State(state): State<HttpState>,
) -> impl IntoResponse {
    let max_response_time_micros = parse_max_response_time_micros(&query_params);
    debug!(
        peer_addr = %peer_addr,
        query_params = ?query_params,
        "handling ping request"
    );

    let ping_sent_time = Instant::now();
    let _ = state.sender.send(Message::Ping { sender: state.name });

    let mut latencies = HashMap::new();
    let mut receiver = state.sender.subscribe();
    let deadline = tokio::time::Instant::now() + Duration::from_micros(max_response_time_micros);
    let mut modules_responses = 0;
    let number_of_modules = 3;

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, receiver.recv()).await {
            Ok(Ok(Message::Pong { sender, .. })) => {
                let latency_us = ping_sent_time.elapsed().as_micros() as u64;
                latencies.insert(sender.to_string() + "_us", latency_us);
            }
            Ok(Ok(_)) => {}
            Ok(Err(_)) | Err(_) => break,
        }
        modules_responses += 1;
        if modules_responses >= number_of_modules {
            debug!(
                "All {} modules have responded to the ping",
                number_of_modules
            );
            break;
        }
    }

    let response = serde_json::json!({
        "latencies": latencies,
        "total_latency_us": ping_sent_time.elapsed().as_micros() as u64,
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

    #[test]
    fn parse_max_response_time_micros_uses_query_value_or_default() {
        let mut query_params = HashMap::new();
        assert_eq!(parse_max_response_time_micros(&query_params), 500_000);

        query_params.insert("max_response_time_micros".to_string(), "1234".to_string());
        assert_eq!(parse_max_response_time_micros(&query_params), 1234);
    }

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
                log_level: "debug".to_string(),
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
                ws_port: 8085,
                log_level: "debug".to_string(),
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
