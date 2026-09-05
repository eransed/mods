#[cfg(feature = "sensor")]
mod camera;

mod config;
mod http;
mod logging;
mod message;
mod openprotocol;
mod udp_discovery_server;
mod util;
mod ws_client;
mod ws_server;
mod module;

use crate::logging::init_tracing;
use crate::message::Message;
use config::ConfigModule;
use http::HttpModule;
use std::net::IpAddr;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use sysinfo::MemoryRefreshKind;
use sysinfo::Pid;
use sysinfo::System;
use tracing::error;

use clap::Parser;
use tracing::debug;
use tracing::info;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use types::BuildInfo;
use types::Config;
use udp_discovery_server::DiscoveryServer;
use ws_client::WsClient;
use ws_server::WsServer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// Name of the person to greet
  #[arg(short, long, default_value_t = String::from("Unknown"))]
  name: String,

  /// Number of times to greet
  #[arg(short, long, default_value_t = 1)]
  count: u8,
}

fn init_tracing_guard(config: &Config) -> WorkerGuard {
  init_tracing(config)
}

async fn run_openprotocol(
  mut config_rx: tokio::sync::watch::Receiver<Config>,
  mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
  state_sender: tokio::sync::broadcast::Sender<Message>,
) {
  loop {
    let activated_clients: Vec<_> = config_rx
      .borrow()
      .open_protocol_configs
      .iter()
      .filter(|c| c.activated.value)
      .cloned()
      .collect();

    let mut clients = tokio::task::JoinSet::new();
    for client_config in &activated_clients {
      info!("Starting OpenProtocol client for: {}", client_config.name.value);
      clients.spawn(run_openprotocol_client(client_config.clone(), state_sender.clone()));
    }

    tokio::select! {
      result = config_rx.changed() => {
        publish_openprotocol_disconnects(&activated_clients, &state_sender, "configuration changed");
        clients.abort_all();
        while clients.join_next().await.is_some() {}
        if result.is_err() {
          return;
        }
      }
      _result = shutdown_rx.changed() => {
        publish_openprotocol_disconnects(&activated_clients, &state_sender, "shutdown requested");
        clients.abort_all();
        while clients.join_next().await.is_some() {}
        return;
      }
      result = clients.join_next(), if !clients.is_empty() => {
        if let Some(Err(error)) = result {
          error!(?error, "OpenProtocol client task stopped unexpectedly");
        }
      }
    }
  }
}

fn publish_openprotocol_disconnects(
  configs: &[types::OpenProtocolClientConfig],
  state_sender: &tokio::sync::broadcast::Sender<Message>,
  error: &str,
) {
  for config in configs {
    let _ = state_sender.send(Message::OpenProtocolState(crate::message::OpenProtocolState {
      name: config.name.value.clone(),
      ip: config.ip.value.clone(),
      port: config.port.value,
      connected: false,
      ping_ms: None,
      error: Some(error.to_string()),
    }));
  }
}

async fn run_openprotocol_client(
  client_config: types::OpenProtocolClientConfig,
  state_sender: tokio::sync::broadcast::Sender<Message>,
) {
  loop {
    match openprotocol::client::client(&client_config, state_sender.clone()).await {
      Ok(()) => info!("OpenProtocol client '{}' stopped successfully", client_config.name.value),
      Err(error) => error!(%error, "OpenProtocol client '{}' error", client_config.name.value),
    }

    tokio::time::sleep(Duration::from_millis(client_config.reconnect_delay_ms.value)).await;
  }
}

pub fn build_info() -> BuildInfo {
  serde_json::from_str(include_str!("../build_info.json")).unwrap_or_default()
}

pub fn version() -> String {
  let bi = build_info();
  format!("{}-{}-{}-{}", bi.cargo_pkg_version, bi.git_hash, bi.build_type, bi.target_arch)
}

#[tokio::main]
async fn main() {
  let args = Args::parse();
  let main_start = Instant::now();
  let (broadcast_sender, _) = tokio::sync::broadcast::channel(16);
  let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
  let (config_request_tx, config_request_rx) = tokio::sync::mpsc::unbounded_channel();
  let (discovery_tx, discovery_rx) = tokio::sync::mpsc::unbounded_channel();

  let (config_module, config_rx) = ConfigModule::new(broadcast_sender.clone(), config_request_rx);
  let initial_config = config_module.config().clone();
  let _guard = init_tracing_guard(&initial_config);

  info!("Starting MODS server with configuration: {:#?}", initial_config);

  info!("Hello {}, count={}", args.name, args.count);
  let bi = build_info();
  debug!("Build info:\n{:#?}", bi);
  info!("Version        : {} ({:.1?})", version(), main_start.elapsed());
  info!("Rust version   : {}", bi.rustc_version);
  info!("Node version   : {}", bi.node_version);
  #[cfg(feature = "sensor")]
  {
    info!("OpenCV version : {}", bi.opencv_version);
  }
  #[cfg(not(feature = "sensor"))]
  {
    warn!("Compiled without sensor support");
  }

  info!(
    "Debug size     : {} KB ({:.1} MB)",
    bi.binary_debug_size_kb,
    bi.binary_debug_size_kb as f32 / 1000 as f32
  );
  info!(
    "Release size   : {} KB ({:.1} MB)",
    bi.binary_release_size_kb,
    bi.binary_release_size_kb as f32 / 1000 as f32
  );
  info!(
    "js size        : {} KB ({:.1} MB)",
    bi.main_js_size_kb,
    bi.main_js_size_kb as f32 / 1000 as f32
  );

  let mut sys = System::new_all();
  sys.refresh_all();

  let b2gb = |b: u64| b as f32 / 1024.0 / 1024.0 / 1024.0;
  let b2mb = |b: u64| b as f32 / 1024.0 / 1024.0;
  let free_memory = sys.total_memory() - sys.used_memory();
  let free_swap = sys.total_swap() - sys.used_swap();

  // RAM and swap information:
  info!("Total memory: {} bytes ({:.1}GB)", sys.total_memory(), b2gb(sys.total_memory()));
  info!("Used memory : {} bytes ({:.1}GB)", sys.used_memory(), b2gb(sys.used_memory()));
  info!("Free memory : {} bytes ({:.1}GB)", free_memory, b2gb(free_memory));

  info!("Total swap  : {} bytes ({:.1}GB)", sys.total_swap(), b2gb(sys.total_swap()));
  info!("Used swap   : {} bytes ({:.1}GB)", sys.used_swap(), b2gb(sys.used_swap()));
  info!("Free swap   : {} bytes ({:.1}GB)", free_swap, b2gb(free_swap));

  // Display system information:
  info!("System name:             {:?}", System::name());
  info!("System kernel version:   {:?}", System::kernel_version());
  info!("System OS version:       {:?}", System::os_version());
  info!("System host name:        {:?}", System::host_name());

  info!("Number of cores: {}", sys.cpus().len());

  let pid = Pid::from_u32(std::process::id());

  info!("Process ID: {}", pid);
  let pid_array = [pid];

  let sdrxsys = shutdown_rx.clone();
  let sysbrd = broadcast_sender.clone();
  let sys_thread_handle = thread::Builder::new().name("SysInfo".to_string()).spawn(move || {

    loop {
      let query_start = Instant::now();
      if *sdrxsys.borrow() {
        info!("System loop shutdown requested");
        break;
      }
      sys.refresh_cpu_usage();
      sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
      sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&pid_array),
        false,
        sysinfo::ProcessRefreshKind::nothing().with_memory().with_cpu(),
      );
      let cpu_percent = sys.global_cpu_usage();
      let ram_percent = (100 * sys.used_memory() / sys.total_memory()) as f32;
      let ram_use_mb = b2mb(sys.used_memory());
      let ram_tot_mb = b2mb(sys.total_memory());
      let p = sys.process(pid).unwrap();
      let pid_mem_bytes = p.memory();
      let pid_mem_use_mb = b2mb(pid_mem_bytes);
      let pid_acc_cpu_time_ms = p.accumulated_cpu_time();
      let pid_cpu_percent = p.cpu_usage();

      sysbrd
        .send(Message::SystemStatus { cpu_percent, ram_percent, pid_mem_bytes })
        .expect("Failed to send system status");

      debug!(
        "[{:.1?}] -- CPU: {:.1}%   RAM: {:.1}% ({:.0}MB / {:.0}MB)   CPU[{}]: {:.2}%   MEM: {:.1}MB   ACC: {:.1}ms   RUNTIME: {:.1?}",
        query_start.elapsed(),
        cpu_percent,
        ram_percent,
        ram_use_mb,
        ram_tot_mb,
        pid,
        pid_cpu_percent,
        pid_mem_use_mb,
        pid_acc_cpu_time_ms,
        main_start.elapsed()
      );
      let sys_status_delay_ms = 1000;
      std::thread::sleep(std::time::Duration::from_millis(sys_status_delay_ms));
    }
  });

  let cam_thread_handle: Option<std::thread::JoinHandle<()>>;

  #[cfg(not(feature = "sensor"))]
  {
    cam_thread_handle = None;
  }

  #[cfg(feature = "sensor")]
  {
    let cam_brdcast = broadcast_sender.clone();
    let sdrxcam = shutdown_rx.clone();
    let mut cam_config_rx = config_rx.clone();
    cam_thread_handle = Some(std::thread::spawn(move || {
      loop {
        let config = cam_config_rx.borrow().clone();
        // Start the first configured camera only when it is enabled.
        if config.camera_configs.first().map(|camera| camera.enable_camera.value).unwrap_or(false) {
          info!("Starting camera thread");
          camera::camera_start(cam_brdcast.clone(), sdrxcam.clone(), cam_config_rx.clone());
          warn!("Camera returned");
        } else {
          warn!("Camera skipped");
        }
        if *sdrxcam.borrow() {
          break;
        }
        while !cam_config_rx.has_changed().unwrap_or(false) {
          std::thread::sleep(Duration::from_millis(100));
          if *sdrxcam.borrow() {
            return;
          }
        }
        let _ = cam_config_rx.borrow_and_update();
      }
    }));
  }

  // Capture the configured server ports for startup and discovery registration.
  let http_port = initial_config.general_config.http_port.value;
  let ws_port = initial_config.general_config.ws_port.value;
  let ws_client = WsClient::new(format!("ws://127.0.0.1:{}", ws_port));
  let ws_client_config_rx = config_rx.clone();
  let ws_client_shutdown_rx = shutdown_rx.clone();

  // Initialize UDP Discovery Server
  let node_name = hostname::get()
    .ok()
    .and_then(|h| h.into_string().ok())
    .unwrap_or_else(|| "mods-server".to_string());
  let local_ip = IpAddr::from([127, 0, 0, 1]);
  let system_type = "mods-server".to_string();

  let discovery_server = match DiscoveryServer::new(
    node_name,
    local_ip,
    http_port,
    system_type,
    broadcast_sender.clone(),
    discovery_rx,
  )
  .await
  {
    Ok(server) => Ok(server),
    Err(e) => {
      warn!("Failed to initialize UDP Discovery Server: {}", e);
      Err(e)
    }
  };

  let discovery_config_rx = config_rx.clone();
  let discovery_shutdown_rx = shutdown_rx.clone();
  tokio::spawn(async move {
    match discovery_server {
      Ok(ds) => {
        ds.run(discovery_config_rx, discovery_shutdown_rx).await;
      },
      Err(e) => {
        error!("Failed to start discovery server: {}", e);
      }
    }
  });

  tokio::spawn(openprotocol::mock_server::run(shutdown_rx.clone()));

  tokio::spawn(run_openprotocol(config_rx.clone(), shutdown_rx.clone(), broadcast_sender.clone()));

  tokio::spawn(async move {
    config_module.run().await;
  });

  let http_config_rx = config_rx.clone();
  let http_shutdown_rx = shutdown_rx.clone();
  let http_sender = broadcast_sender.clone();
  let http_shutdown = shutdown_tx.clone();
  let http_config_request = config_request_tx.clone();
  let http_discovery_tx = discovery_tx.clone();
  tokio::spawn(async move {
    let mut config_rx = http_config_rx;
    let mut shutdown_rx = http_shutdown_rx;
    loop {
      let config = config_rx.borrow().clone();
      let host = if config.general_config.allow_remote_connections.value {
        [0, 0, 0, 0]
      } else {
        [127, 0, 0, 1]
      };
      let addr = std::net::SocketAddr::from((host, config.general_config.http_port.value));
      let module = HttpModule::new(
        "http",
        http_sender.clone(),
        http_shutdown.clone(),
        http_config_request.clone(),
        http_discovery_tx.clone(),
      );
      let mut task = tokio::spawn(module.run(addr));
      tokio::select! {
        result = &mut task => {
          if let Ok(Err(err)) = result {
            error!(error = ?err, "failed to start http server");
          }
        }
        result = config_rx.changed() => {
          // Wait for the HTTP listener task to release its socket before restarting it.
          task.abort();
          let _ = task.await;
          if result.is_err() { break; }
        }
        result = shutdown_rx.changed() => {
          // Wait for the HTTP listener task to release its socket before shutting down.
          task.abort();
          let _ = task.await;
          if result.is_err() || *shutdown_rx.borrow() { break; }
        }
      }
    }
  });

  let ws_config_rx = config_rx.clone();
  let ws_shutdown_rx = shutdown_rx.clone();
  let ws_sender = broadcast_sender.clone();
  tokio::spawn(async move {
    let mut config_rx = ws_config_rx;
    let mut shutdown_rx = ws_shutdown_rx;
    loop {
      let config = config_rx.borrow().clone();
      let host = if config.general_config.allow_remote_connections.value {
        [0, 0, 0, 0]
      } else {
        [127, 0, 0, 1]
      };
      let addr = std::net::SocketAddr::from((host, config.general_config.ws_port.value));
      let module = WsServer::new("ws_server", ws_sender.clone());
      let mut task = tokio::spawn(module.run(addr));
      tokio::select! {
        result = &mut task => {
          if let Ok(Err(err)) = result {
            error!(error = ?err, "failed to start websocket server");
          }
        }
        result = config_rx.changed() => {
          // Wait for the websocket listener task to release its socket before rebinding.
          task.abort();
          let _ = task.await;
          if result.is_err() { break; }
        }
        result = shutdown_rx.changed() => {
          // Wait for the websocket listener task to release its socket before shutting down.
          task.abort();
          let _ = task.await;
          if result.is_err() || *shutdown_rx.borrow() { break; }
        }
      }
    }
  });



  tokio::spawn(async move {
    tokio::time::sleep(Duration::from_millis(200)).await;
    ws_client.run(ws_client_config_rx, ws_client_shutdown_rx).await;
  });

  info!("http server listening at: {}", http_port);
  info!("websocket server listening at: {}", ws_port);

  info!("Current working directory: {}", std::env::current_dir().unwrap().display());

  // handle stop signals and shutdown

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

  info!("Sending shutdown signal");
  let _ = shutdown_tx.send(true);

  info!("Waiting for sys thread to stop...");
  sys_thread_handle.unwrap().join().expect("Failed to join sys thread");

  match cam_thread_handle {
    Some(handle) => {
      info!("Waiting for camera thread to stop...");
      handle.join().expect("Failed to join camera thread");
    }
    None => {
      info!("Camera thread was not started");
    }
  }

  info!("shutting down after {:.1?}", main_start.elapsed());
}

pub struct SystemInfo {
  pub total_memory: u64,
  pub used_memory: u64,
  pub total_swap: u64,
  pub used_swap: u64,
}
