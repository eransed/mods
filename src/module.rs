use std::{
  env, fs,
  path::{Path, PathBuf},
};

use crate::message::Message;
use tokio::sync::{
  broadcast::{Receiver, Sender},
  watch,
};
use tracing::{error, info};
use types::Config;

pub struct Module {
  shutdown_watcher: (watch::Sender<bool>, watch::Receiver<bool>),
  message_bus: (Sender<Message>, Receiver<Message>),
  config_watcher: (watch::Sender<Config>, watch::Receiver<Config>),
  config: Config,
}

impl Default for Module {
  fn default() -> Self {
    let config = load_config_from_path(&config_path());
    Self {
      shutdown_watcher: tokio::sync::watch::channel(false),
      message_bus: tokio::sync::broadcast::channel(16),
      config_watcher: watch::channel(config.clone()),
      config,
    }
  }
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
  let contents = serde_json::to_string_pretty(config).expect("config failed to serialize");
  if let Some(parent) = path.parent()
    && !parent.as_os_str().is_empty()
  {
    fs::create_dir_all(parent)?;
  }
  info!("Saving config to {}: {:#?}", path.display(), config);
  fs::write(path, contents)
}
