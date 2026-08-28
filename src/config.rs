use crate::{logging::set_log_level, message::Message};
use std::{
  env, fs,
  path::{Path, PathBuf},
};
use tokio::sync::{
  broadcast::{Receiver, Sender},
  mpsc::UnboundedReceiver,
  watch,
};
use tracing::{debug, error, info, warn};
use types::Config;

pub enum ConfigRequest {
  Get { requester: &'static str, response: tokio::sync::oneshot::Sender<Config> },
  Set { requester: &'static str, config: Config, response: tokio::sync::oneshot::Sender<Config> },
  Reset { requester: &'static str, response: tokio::sync::oneshot::Sender<Config> },
}

pub struct ConfigModule {
  receiver: Receiver<Message>,
  sender: Sender<Message>,
  request_receiver: UnboundedReceiver<ConfigRequest>,
  config: Config,
  config_sender: watch::Sender<Config>,
}

fn config_path() -> PathBuf {
  env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("config.json")
}

fn load_config_from_path(path: &Path) -> Config {
  match fs::read_to_string(path) {
    Ok(contents) => match serde_json::from_str::<Config>(&contents) {
      Ok(config) => config,
      Err(err) => {
        error!(error = ?err, path = ?path, "failed to parse config.json, using default config");
        let default = Config::default();
        if let Err(write_err) = save_config_to_path(&default, path) {
          error!(error = ?write_err, path = ?path, "failed to write default config");
        }
        default
      }
    },
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
      let default = Config::default();
      if let Err(write_err) = save_config_to_path(&default, path) {
        error!(error = ?write_err, path = ?path, "failed to create default config file");
      }
      default
    }
    Err(err) => {
      error!(error = ?err, path = ?path, "failed to read config.json, using default config");
      Config::default()
    }
  }
}

fn save_config_to_path(config: &Config, path: &Path) -> std::io::Result<()> {
  let contents = serde_json::to_string_pretty(config).expect("config should serialize");
  if let Some(parent) = path.parent()
    && !parent.as_os_str().is_empty()
  {
    fs::create_dir_all(parent)?;
  }
  info!("Saving config to {}: {:#?}", path.display(), config);
  fs::write(path, contents)
}

impl ConfigModule {
  pub fn config(&self) -> &Config {
    &self.config
  }

  pub fn new(
    sender: Sender<Message>,
    request_receiver: UnboundedReceiver<ConfigRequest>,
  ) -> (Self, watch::Receiver<Config>) {
    let receiver = sender.subscribe();
    let config = load_config_from_path(&config_path());
    let (config_sender, config_receiver) = watch::channel(config.clone());
    (
      Self { receiver, sender: sender.clone(), request_receiver, config, config_sender },
      config_receiver,
    )
  }

  pub async fn run(mut self) {
    loop {
      debug!("config run loop iter");
      tokio::select! {
          maybe_request = self.request_receiver.recv() => match maybe_request {
              Some(request) => match request {
                  ConfigRequest::Get { requester, response } => {
                      debug!(requester, "get config");
                      let _ = response.send(self.config.clone());
                  }
                  ConfigRequest::Set { requester, config, response } => {
                      debug!(requester, "set config");
                      self.config = config.clone();
                      let _ = self.config_sender.send(config.clone());
                      // Apply the configured log level after accepting a new configuration.
                      set_log_level(&self.config.logging_config.log_level.value);
                      if let Err(err) = save_config_to_path(&self.config, &config_path()) {
                          error!(error = ?err, "failed to persist config to config.json");
                      }
                      let _ = response.send(self.config.clone());
                  }
                  ConfigRequest::Reset { requester, response } => {
                      debug!(requester, "reset config");
                      self.config = Config::default();
                      let _ = self.config_sender.send(self.config.clone());
                      // Apply the default log level after resetting the configuration.
                      set_log_level(&self.config.logging_config.log_level.value);
                      if let Err(err) = save_config_to_path(&self.config, &config_path()) {
                          error!(error = ?err, "failed to persist default config to config.json");
                      }
                      let _ = response.send(self.config.clone());
                  }
              },
              None => {
                  warn!("request channel closed");
                  break;
              }
          },
          result = self.receiver.recv() => match result {
              Ok(Message::Broadcast { sender, body }) => {
                  debug!("broadcast received: {} bytes from {}", body.len(), sender);
              }
              Ok(Message::Ping { sender }) => {
                  debug!("ping received from {}", sender);
                  let _ = self.sender.send(Message::Pong {
                      sender: "config",
                  });
              }
              Ok(Message::Pong { sender }) => {
                  debug!("pong received from {}", sender);
              }
              Ok(Message::Discovery(event)) => {
                  debug!("discovery event received: {:?}", event);
              }
              Ok(_) => {
                debug!("Empty broadcast");
              }
              Err(_) => {
                  error!("broadcast channel closed");
                  break;
              }
          },
      }
    }

    warn!("shutting down");
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
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
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
  fn defaults_to_info_log_level() {
    let config = Config::default();
    assert_eq!(config.logging_config.log_level.value, "info");
    assert_eq!(config.volumes.len(), 3);
    assert_eq!(config.volumes[0].name, "bolt_1");
    assert_eq!(config.volumes[0].enter_radius, 5.0);
    assert_eq!(config.volumes[0].exit_radius, 7.5);
  }

  #[test]
  fn saves_and_loads_config_from_disk() {
    let path = temp_config_path();
    // Change representative nested values while preserving their metadata.
    let mut config = Config::default();
    config.general_config.http_port.value = 9000;
    config.general_config.ws_port.value = 9001;
    config.general_config.allow_remote_connections.value = false;
    assert!(!config.camera_configs.is_empty());
    let camera = &mut config.camera_configs[0];
    camera.opencv_display.value = true;
    camera.angle_filter.value = 5;
    camera.min_decision_margin.value = 25.0;
    camera.device_index.value = 1;
    camera.camera_send_image.value = false;

    save_config_to_path(&config, &path).unwrap();
    let loaded = load_config_from_path(&path);

    assert_eq!(loaded, config);
    let _ = fs::remove_file(path);
  }
}
