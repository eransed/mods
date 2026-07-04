use crate::message::Message;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tokio::sync::{
    broadcast::{Receiver, Sender},
    mpsc::UnboundedReceiver,
};
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
    ResetConfig {
        requester: &'static str,
        response: tokio::sync::oneshot::Sender<Config>,
    },
}

pub struct ConfigModule {
    receiver: Receiver<Message>,
    sender: Sender<Message>,
    request_receiver: UnboundedReceiver<ConfigRequest>,
    config: Config,
}

fn config_path() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.json")
}

fn load_config_from_path(path: &Path) -> Config {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<Config>(&contents) {
            Ok(config) => config,
            Err(err) => {
                info!(error = ?err, path = ?path, "failed to parse config.json, using default config");
                let default = Config::default();
                if let Err(write_err) = save_config_to_path(&default, path) {
                    info!(error = ?write_err, path = ?path, "failed to write default config");
                }
                default
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let default = Config::default();
            if let Err(write_err) = save_config_to_path(&default, path) {
                info!(error = ?write_err, path = ?path, "failed to create default config file");
            }
            default
        }
        Err(err) => {
            info!(error = ?err, path = ?path, "failed to read config.json, using default config");
            Config::default()
        }
    }
}

fn save_config_to_path(config: &Config, path: &Path) -> std::io::Result<()> {
    let contents = serde_json::to_string_pretty(config).expect("config should serialize");
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)
}

impl ConfigModule {
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn new(
        sender: Sender<Message>,
        request_receiver: UnboundedReceiver<ConfigRequest>,
    ) -> Self {
        let receiver = sender.subscribe();
        let config = load_config_from_path(&config_path());
        Self {
            receiver,
            sender: sender.clone(),
            request_receiver,
            config,
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
                            self.config = config.clone();
                            if let Err(err) = save_config_to_path(&self.config, &config_path()) {
                                info!(error = ?err, "failed to persist config to config.json");
                            }
                            let _ = response.send(self.config.clone());
                        }
                        ConfigRequest::ResetConfig { requester, response } => {
                            info!(requester, "config reset config request");
                            self.config = Config::default();
                            if let Err(err) = save_config_to_path(&self.config, &config_path()) {
                                info!(error = ?err, "failed to persist default config to config.json");
                            }
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
                    Ok(Message::Ping { timestamp, sender }) => {
                        info!(timestamp, "config ping received from {}", sender);
                        let _ = self.sender.send(Message::Pong {
                            sender: "config",
                            timestamp,
                        });
                    }
                    Ok(Message::Pong { timestamp, sender }) => {
                        info!(timestamp, "config pong received from {}", sender);
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

#[cfg(test)]
mod tests {
    use super::{Config, load_config_from_path, save_config_to_path};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_config_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mods-config-test-{nanos}.json"))
    }

    #[test]
    fn loads_default_config_and_creates_file_when_missing() {
        let path = temp_config_path();
        let config = load_config_from_path(&path);

        assert_eq!(config, Config::default());
        assert!(path.exists());

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("http_port"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_and_loads_config_from_disk() {
        let path = temp_config_path();
        let config = Config {
            http_port: 9000,
            ws_port: 9001,
        };

        save_config_to_path(&config, &path).unwrap();
        let loaded = load_config_from_path(&path);

        assert_eq!(loaded, config);
        let _ = fs::remove_file(path);
    }
}
