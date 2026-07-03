use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast::{Receiver, Sender}, mpsc::UnboundedReceiver};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub ws_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_port: 8080,
            ws_port: 8081,
        }
    }
}

#[derive(Clone)]
pub enum Message {
    Broadcast { sender: &'static str, body: String },
}

pub enum ConfigRequest {
    GetConfig {
        requester: &'static str,
        response: tokio::sync::oneshot::Sender<Config>,
    },
    SetConfig {
        requester: &'static str,
        config: Config,
        response: tokio::sync::oneshot::Sender<Config>,
    },
}

pub struct ConfigModule {
    receiver: Receiver<Message>,
    request_receiver: UnboundedReceiver<ConfigRequest>,
    config: Config,
}

impl ConfigModule {
    pub fn new(
        sender: Sender<Message>,
        request_receiver: UnboundedReceiver<ConfigRequest>,
    ) -> Self {
        let receiver = sender.subscribe();
        Self {
            receiver,
            request_receiver,
            config: Config::default(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                maybe_request = self.request_receiver.recv() => match maybe_request {
                    Some(request) => match request {
                        ConfigRequest::GetConfig { requester, response } => {
                            info!(requester, "config get config request");
                            let _ = response.send(self.config.clone());
                        }
                        ConfigRequest::SetConfig { requester, config, response } => {
                            info!(requester, "config set config request");
                            self.config = config;
                            let _ = response.send(self.config.clone());
                        }
                    },
                    None => {
                        info!("config request channel closed");
                        break;
                    }
                },
                result = self.receiver.recv() => match result {
                    Ok(Message::Broadcast { sender, body }) => {
                        info!(sender, body, "config broadcast received");
                    }
                    Err(_) => {
                        info!("config broadcast channel closed");
                        break;
                    }
                },
            }
        }

        info!("config shutting down");
    }
}

impl Drop for ConfigModule {
    fn drop(&mut self) {
        info!("config dropping and shutting down");
    }
}
