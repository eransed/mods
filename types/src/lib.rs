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
pub struct ConfigProperty<T> {
  pub value: T,
  pub default_value: T,
  pub added_version: String,
  pub description: String,
  pub hide: bool,
  pub deprecated_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
  pub log_level: ConfigProperty<String>,
  pub max_lines_per_file: ConfigProperty<usize>,
  pub max_log_file_to_keep: ConfigProperty<usize>,
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self {
      log_level: ConfigProperty {
        value: "info".to_string(),
        default_value: "info".to_string(),
        added_version: "1.0.0".to_string(),
        description: "The log level".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      max_lines_per_file: ConfigProperty {
        value: 10_000,
        default_value: 10_000,
        added_version: "1.0.0".to_string(),
        description: "The maximum number of lines per log file".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      max_log_file_to_keep: ConfigProperty {
        value: 100,
        default_value: 100,
        added_version: "1.0.0".to_string(),
        description: "The maximum number of log files to keep".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MidConfig {
  pub rev: u16,
  pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenProtocolClientConfig {
  pub activated: ConfigProperty<bool>,
  pub name: ConfigProperty<String>,
  pub ip: ConfigProperty<String>,
  pub port: ConfigProperty<u16>,
  pub keep_alive_time_ms: ConfigProperty<u64>,
  pub reconnect_delay_ms: ConfigProperty<u64>,
  pub mid_0001_config: MidConfig,
}

impl Default for OpenProtocolClientConfig {
  fn default() -> Self {
    Self {
      activated: ConfigProperty {
        value: false,
        default_value: false,
        added_version: "1.0.0".to_string(),
        description: "Whether the client is activated".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      name: ConfigProperty {
        value: "default".to_string(),
        default_value: "default".to_string(),
        added_version: "1.0.0".to_string(),
        description: "The name of the client".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      ip: ConfigProperty {
        value: "127.0.0.1".to_string(),
        default_value: "127.0.0.1".to_string(),
        added_version: "1.0.0".to_string(),
        description: "The IP address of the client".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      port: ConfigProperty {
        value: 4545,
        default_value: 4545,
        added_version: "1.0.0".to_string(),
        description: "The port of the client".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      keep_alive_time_ms: ConfigProperty {
        value: 7500,
        default_value: 7500,
        added_version: "1.0.0".to_string(),
        description: "The keep-alive time in milliseconds".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      reconnect_delay_ms: ConfigProperty {
        value: 5000,
        default_value: 5000,
        added_version: "1.0.0".to_string(),
        description: "The reconnect delay in milliseconds".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      mid_0001_config: MidConfig { rev: 6, active: true },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraConfig {
  pub name: ConfigProperty<String>,
  pub enable_camera: ConfigProperty<bool>,
  pub opencv_display: ConfigProperty<bool>,
  pub angle_filter: ConfigProperty<usize>,
  pub min_decision_margin: ConfigProperty<f32>,
  pub device_index: ConfigProperty<usize>,
  pub device_width: ConfigProperty<f64>,
  pub camera_fetch_delay_ms: ConfigProperty<u64>,
  pub camera_send_image: ConfigProperty<bool>,
  pub camera_send_image_resize_factor: ConfigProperty<f64>,
}

impl Default for CameraConfig {
  fn default() -> Self {
    Self {
      name: ConfigProperty {
        value: "default".to_string(),
        default_value: "default".to_string(),
        added_version: "1.0.0".to_string(),
        description: "The name of the camera".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      enable_camera: ConfigProperty {
        value: true,
        default_value: true,
        added_version: "1.0.0".to_string(),
        description: "Whether to enable the camera".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      opencv_display: ConfigProperty {
        value: false,
        default_value: false,
        added_version: "1.0.0".to_string(),
        description: "Whether to display the OpenCV window".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      angle_filter: ConfigProperty {
        value: 3,
        default_value: 3,
        added_version: "1.0.0".to_string(),
        description: "The number of angles to filter".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      min_decision_margin: ConfigProperty {
        value: 20.0,
        default_value: 20.0,
        added_version: "1.0.0".to_string(),
        description: "The minimum decision margin".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      device_index: ConfigProperty {
        value: 0,
        default_value: 0,
        added_version: "1.0.0".to_string(),
        description: "The index of the camera device to use".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      device_width: ConfigProperty {
        value: 1920 as f64,
        default_value: 1920 as f64,
        added_version: "1.0.0".to_string(),
        description: "The width of the camera device".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_fetch_delay_ms: ConfigProperty {
        value: 0,
        default_value: 0,
        added_version: "1.0.0".to_string(),
        description: "The delay for fetching camera images".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_send_image: ConfigProperty {
        value: true,
        default_value: true,
        added_version: "1.0.0".to_string(),
        description: "Whether to send the camera image".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_send_image_resize_factor: ConfigProperty {
        value: 0.4,
        default_value: 0.4,
        added_version: "1.0.0".to_string(),
        description: "The resize factor for the camera image".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
  pub _bool_property: ConfigProperty<bool>,
  pub _string_property: ConfigProperty<String>,
  pub _number_property: ConfigProperty<u32>,
  pub http_port: ConfigProperty<u16>,
  pub ws_port: ConfigProperty<u16>,
  pub allow_remote_connections: ConfigProperty<bool>,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    Self {
      _bool_property: ConfigProperty {
        value: true,
        default_value: true,
        added_version: "0.0.0".to_string(),
        description: "An example boolean property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      _string_property: ConfigProperty {
        value: "default".to_string(),
        default_value: "default".to_string(),
        added_version: "0.0.0".to_string(),
        description: "An example string property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      _number_property: ConfigProperty {
        value: 42,
        default_value: 42,
        added_version: "0.0.0".to_string(),
        description: "An example number property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      http_port: ConfigProperty {
        value: 8123,
        default_value: 8123,
        added_version: "1.0.0".to_string(),
        description: "Port that the http server shall listen on".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      ws_port: ConfigProperty {
        value: 8124,
        default_value: 8124,
        added_version: "1.0.0".to_string(),
        description: "Port that the websocket server shall listen on".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      allow_remote_connections: ConfigProperty {
        value: true,
        default_value: true,
        added_version: "1.0.0".to_string(),
        description: "Whether to allow remote connections".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
  pub general_config: GeneralConfig,
  pub logging_config: LoggingConfig,
  pub camera_configs: Vec<CameraConfig>,
  pub open_protocol_configs: Vec<OpenProtocolClientConfig>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      general_config: GeneralConfig::default(),
      logging_config: LoggingConfig::default(),
      camera_configs: vec![CameraConfig::default()],
      open_protocol_configs: vec![OpenProtocolClientConfig::default()],
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
