use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::{
    broadcast::Sender as BroadcastSender, mpsc::UnboundedSender, watch::Sender as WatchSender,
};
use tracing::info;

use crate::config::{Config, ConfigRequest, Message};

#[derive(Clone)]
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

impl Drop for HttpModule {
    fn drop(&mut self) {
        info!("http dropping and shutting down");
    }
}
