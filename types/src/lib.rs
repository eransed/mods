use std::usize;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BuildInfo {
  pub binary_release_size_kb: u64,
  pub binary_debug_size_kb: u64,
  pub index_html_size_kb: u64,
  pub main_js_size_kb: u64,
  pub main_css_size_kb: u64,
  pub cargo_pkg_name: String,
  pub cargo_pkg_version: String,
  pub git_branch: String,
  pub git_hash: String,
  pub git_date: String,
  pub build_time_utc: String,
  pub build_type: String,
  pub build_uname: String,
  pub rustc_version: String,
  pub git_version: String,
  pub docker_version: String,
  pub node_version: String,
  pub npm_version: String,
  pub quicktype_version: String,
  pub opencv_version: String,
  pub target_arch: String,
  pub target_avx2: bool,
  pub target_neon: bool,
  pub windows: bool,
  pub compiled_with_sensor_support: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
  pub log_level: String,
  pub max_lines_per_file: usize,
  pub max_log_file_to_keep: usize,
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self { log_level: "info".to_string(), max_lines_per_file: 10_000, max_log_file_to_keep: 100 }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MidConfig {
  pub rev: u16,
  pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenProtocolClientConfig {
  pub activated: bool,
  pub name: String,
  pub ip: String,
  pub port: u16,
  pub keep_alive_time_ms: u64,
  pub reconnect_delay_ms: u64,
  pub mid_0001_config: MidConfig,
}

impl Default for OpenProtocolClientConfig {
  fn default() -> Self {
    Self {
      activated: false,
      name: "default".to_string(),
      ip: "127.0.0.1".to_string(),
      port: 4545,
      keep_alive_time_ms: 7500,
      reconnect_delay_ms: 5000,
      mid_0001_config: MidConfig { rev: 6, active: true },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenProtocolConfig {
  pub open_protocol_clients: Vec<OpenProtocolClientConfig>,
}

impl Default for OpenProtocolConfig {
  fn default() -> Self {
    Self { open_protocol_clients: vec![OpenProtocolClientConfig::default()] }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigProperty<T> {
  pub value: T,
  pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
  pub http_port: ConfigProperty<u16>,
  pub ws_port: u16,
  pub allow_remote_connections: bool,
  pub enable_camera: bool,
  pub opencv_display: bool,
  pub angle_filter: usize,
  pub min_decision_margin: f32,
  pub device_index: i32,
  pub device_width: f64,
  pub camera_fetch_delay_ms: u64,
  pub camera_send_image: bool,
  pub camera_send_image_resize_factor: f64,
  pub logging_config: LoggingConfig,
  pub open_protocol_config: OpenProtocolConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      http_port: ConfigProperty {
        value: 8123,
        description: "Port that the http server shall listen on".to_string(),
      },
      ws_port: 8124,
      allow_remote_connections: true,
      enable_camera: true,
      opencv_display: false,
      angle_filter: 3,
      min_decision_margin: 20.0,
      device_index: 0,
      device_width: 1920 as f64,
      camera_fetch_delay_ms: 0,
      camera_send_image: true,
      camera_send_image_resize_factor: 0.4,
      logging_config: LoggingConfig::default(),
      open_protocol_config: OpenProtocolConfig::default(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagPose {
  pub id: usize,
  pub center_image: (f64, f64),
  pub decision_margin: f32,
  pub translation: (f64, f64, f64),
  pub rotation: (f64, f64, f64),
  pub pose_estimation_time_us: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawImageDetection {
  pub tags: Vec<TagPose>,
  pub image_data_base64: String,
  pub image_size: (i32, i32),
  pub native_image_size: (i32, i32),
  pub detection_time_us: u32,
  pub image_encoding_time_us: u32,
  pub send_freq: f32,
}
