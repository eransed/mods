mod config;
mod http;
mod logging;
mod ws_client;
mod ws_server;

use crate::logging::init_tracing;
use config::ConfigModule;
use http::HttpModule;
use std::time::Duration;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use ws_client::WsClient;
use ws_server::WsServer;

fn init_tracing_guard() -> WorkerGuard {
    init_tracing()
}

#[tokio::main]
async fn main() {
    let _guard = init_tracing_guard();
    let (tx, _) = tokio::sync::broadcast::channel(16);

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let (config_request_tx, config_request_rx) = tokio::sync::mpsc::unbounded_channel();

    let config_module = ConfigModule::new(tx.clone(), config_request_rx);
    let ws_server = WsServer::new("ws_server", tx.clone());
    let http_module = HttpModule::new(
        "http",
        tx.clone(),
        shutdown_tx.clone(),
        config_request_tx.clone(),
    );
    let ws_client = WsClient::new("ws://127.0.0.1:8081".to_string());

    tokio::spawn(async move {
        config_module.run().await;
    });

    tokio::spawn(async move {
        let ws_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8081));
        if let Err(err) = ws_server.run(ws_addr).await {
            tracing::error!(error = ?err, "ws_server failed to run websocket server");
        }
    });

    tokio::spawn(async move {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
        if let Err(err) = http_module.run(addr).await {
            tracing::error!(error = ?err, "http failed to run server");
        }
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        ws_client.run().await;
    });

    info!("http server ready at http://127.0.0.1:8080/send");
    info!("websocket server ready at ws://127.0.0.1:8081");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl-c");
        }
        _ = shutdown_rx.changed() => {
            if *shutdown_rx.borrow() {
                info!("received shutdown request");
            }
        }
    }

    info!("shutting down");
}
