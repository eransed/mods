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

use crate::logging::init_tracing;
use crate::message::Message;
use crate::openprotocol::config::MidConfig;
use crate::openprotocol::core::MidName;
use crate::openprotocol::mid_0002::Mid0002;
use config::ConfigModule;
use http::HttpModule;
use tracing::error;
use std::net::IpAddr;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use strum::IntoEnumIterator;
use sysinfo::MemoryRefreshKind;
use sysinfo::Pid;
use sysinfo::System;

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

  let config_module = ConfigModule::new(broadcast_sender.clone(), config_request_rx);
  let initial_config = config_module.config().clone();

  let _guard = init_tracing_guard(&initial_config);
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
    cam_thread_handle = Some(std::thread::spawn(move || {
      if initial_config.enable_camera {
        info!("Starting camera thread");
        camera::camera_start(
          cam_brdcast,
          sdrxcam,
          initial_config.device_index,
          initial_config.device_width,
          initial_config.opencv_display,
          initial_config.angle_filter,
          initial_config.min_decision_margin,
          initial_config.camera_fetch_delay_ms,
          initial_config.camera_send_image,
          initial_config.camera_send_image_resize_factor,
        );
        warn!("Camera returned");
      } else {
        warn!("Camera skipped");
      }
    }));
  }

  let ws_server = WsServer::new("ws_server", broadcast_sender.clone());

  let http_module = HttpModule::new(
    "http",
    broadcast_sender.clone(),
    shutdown_tx.clone(),
    config_request_tx.clone(),
    discovery_tx.clone(),
  );

  let ws_client = WsClient::new(format!("ws://127.0.0.1:{}", initial_config.ws_port));

  let ws_port = initial_config.ws_port;
  let http_port = initial_config.http_port;
  let host = if initial_config.allow_remote_connections { [0, 0, 0, 0] } else { [127, 0, 0, 1] };

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
    http_port.value,
    system_type,
    broadcast_sender.clone(),
    discovery_rx,
  )
  .await
  {
    Ok(server) => server,
    Err(e) => {
      warn!("Failed to initialize UDP Discovery Server: {}", e);
      std::process::exit(1);
    }
  };

  tokio::spawn(async move {
    let c = openprotocol::config::Config {
      ip: String::from("192.168.0.47"),
      port: 4545,
      keep_alive_time_ms: 7500,
      reconnect_delay_ms: 5000,
      mid_0001_config: MidConfig { rev: 6, active: true },
    };

    loop {
      info!("Starts OpenProtcol client...");
      match openprotocol::client::client(&c).await {
        Ok(_) => {
          info!("OpenProtocol client stopped successfully");
          break;
        },
        Err(e) => {
          error!("OpenProtcol client error: {}", e);
          tokio::time::sleep(Duration::from_millis(c.reconnect_delay_ms)).await;
        }
      }
    }
  });

  tokio::spawn(async move {
    config_module.run().await;
  });

  tokio::spawn(async move {
    let ws_addr = std::net::SocketAddr::from((host, ws_port));
    if let Err(err) = ws_server.run(ws_addr).await {
      tracing::error!(error = ?err, "failed to start websocket server");
    }
  });

  tokio::spawn(async move {
    let http_addr = std::net::SocketAddr::from((host, http_port.value));
    if let Err(err) = http_module.run(http_addr).await {
      tracing::error!(error = ?err, "failed to start http server");
    }
  });

  tokio::spawn(async move {
    discovery_server.run().await;
  });

  tokio::spawn(async move {
    tokio::time::sleep(Duration::from_millis(200)).await;
    ws_client.run().await;
  });

  info!("http server listening at: {}", http_port.value);
  info!("websocket server listening at: {}", ws_port);

  info!("Current working directory: {}", std::env::current_dir().unwrap().display());

  let mut mid2 = Mid0002::default();
  mid2.rev1.controller_name = String::from("test");

  info!("{:#?}", mid2);

  info!("Available MIDs:");
  for mid_name in MidName::iter() {
    info!("MID {:04} {}", mid_name as u16, mid_name.as_ref());
  }

  // for i in 0..100_000 {
  //   info!("Testing the logging system with this average length log line: i = {}", i);
  // }
  // info!("Done");

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
