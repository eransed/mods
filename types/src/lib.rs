use std::usize;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
  #[serde(default)]
  pub allowed_values: Option<Vec<String>>,
  #[serde(default)]
  pub input_type: Option<String>,
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
  #[serde(default = "default_log_page_size")]
  pub log_page_size: ConfigProperty<usize>,
}

fn default_log_page_size() -> ConfigProperty<usize> {
  ConfigProperty {
    value: 500,
    default_value: 500,
    allowed_values: None,
    input_type: None,
    added_version: "1.0.0".to_string(),
    description: "The number of log messages shown per page".to_string(),
    hide: false,
    deprecated_version: String::new(),
  }
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self {
      log_level: ConfigProperty {
        value: "info".to_string(),
        default_value: "info".to_string(),
        allowed_values: Some(vec!["trace".to_string(), "debug".to_string(), "info".to_string(), "warn".to_string(), "error".to_string()]),
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The log level".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      max_lines_per_file: ConfigProperty {
        value: 10_000,
        default_value: 10_000,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The maximum number of lines per log file".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      max_log_file_to_keep: ConfigProperty {
        value: 100,
        default_value: 100,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The maximum number of log files to keep".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      log_page_size: default_log_page_size(),
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
        value: true,
        default_value: true,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Whether this client connection is activated".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      name: ConfigProperty {
        value: "default".to_string(),
        default_value: "default".to_string(),
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The name of this connection".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      ip: ConfigProperty {
        value: "127.0.0.1".to_string(),
        default_value: "127.0.0.1".to_string(),
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The IP address of the open protocol server".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      port: ConfigProperty {
        value: 4545,
        default_value: 4545,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The port of the open protocol server to connect to".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      keep_alive_time_ms: ConfigProperty {
        value: 7500,
        default_value: 7500,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The keep-alive time in milliseconds".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      reconnect_delay_ms: ConfigProperty {
        value: 5000,
        default_value: 5000,
        allowed_values: None,
        input_type: None,
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
  pub backend: ConfigProperty<String>,
  pub gstreamer_raw: ConfigProperty<String>,
  pub opencv_display: ConfigProperty<bool>,
  pub angle_filter: ConfigProperty<usize>,
  pub min_decision_margin: ConfigProperty<f32>,
  pub device_index: ConfigProperty<usize>,
  pub device_width: ConfigProperty<f64>,
  pub device_height: ConfigProperty<f64>,
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
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The name of the camera".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      enable_camera: ConfigProperty {
        value: true,
        default_value: true,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Whether to enable the camera".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      backend: ConfigProperty {
        value: "opencv_any".to_owned(),
        default_value: "opencv_any".to_owned(),
        allowed_values: Some(vec!["opencv_any".to_string(), "opencv_gstreamer".to_string()]),
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Backend that shall be used to open the camera device on the host".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      gstreamer_raw: ConfigProperty {
        value: "".to_owned(),
        default_value: "".to_owned(),
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Raw gstreamer pipeline string".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      opencv_display: ConfigProperty {
        value: false,
        default_value: false,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Whether to display the OpenCV window on the host machine".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      angle_filter: ConfigProperty {
        value: 3,
        default_value: 3,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The number of angles to filter".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      min_decision_margin: ConfigProperty {
        value: 20.0,
        default_value: 20.0,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The minimum decision margin".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      device_index: ConfigProperty {
        value: 0,
        default_value: 0,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The index of the camera device to use".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      device_width: ConfigProperty {
        value: 1920 as f64,
        default_value: 1920 as f64,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The width of the camera device".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      device_height: ConfigProperty {
        value: 1080 as f64,
        default_value: 1920 as f64,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The height of the camera device".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_fetch_delay_ms: ConfigProperty {
        value: 0,
        default_value: 0,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "The delay for fetching camera images".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_send_image: ConfigProperty {
        value: true,
        default_value: true,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Whether to send the camera image".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      camera_send_image_resize_factor: ConfigProperty {
        value: 0.4,
        default_value: 0.4,
        allowed_values: None,
        input_type: None,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInterfaceConfig {
  pub notification_position: ConfigProperty<String>,
  pub background_color: ConfigProperty<String>,
  pub foreground_color: ConfigProperty<String>,
  pub accent_color: ConfigProperty<String>,
}

impl Default for UserInterfaceConfig {
  fn default() -> Self {
    Self {
      notification_position: ConfigProperty {
        value: "bottom_left".to_string(),
        default_value: "bottom_left".to_string(),
        allowed_values: Some(vec!["top_left".to_string(), "top_right".to_string(), "bottom_left".to_string(), "bottom_right".to_string()]),
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Screen corner where notifications are displayed".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      background_color: color_property("#161a1eff", "Background color"),
      foreground_color: color_property("#f4f6f8ff", "Foreground color"),
      accent_color: color_property("#ebcd26ff", "Accent color"),
    }
  }
}

fn color_property(value: &str, description: &str) -> ConfigProperty<String> {
  ConfigProperty {
    value: value.to_string(),
    default_value: value.to_string(),
    allowed_values: None,
    input_type: Some("color".to_string()),
    added_version: "1.0.0".to_string(),
    description: description.to_string(),
    hide: false,
    deprecated_version: String::new(),
  }
}

impl Default for GeneralConfig {
  fn default() -> Self {
    Self {
      _bool_property: ConfigProperty {
        value: true,
        default_value: true,
        allowed_values: None,
        input_type: None,
        added_version: "0.0.0".to_string(),
        description: "An example boolean property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      _string_property: ConfigProperty {
        value: "default".to_string(),
        default_value: "default".to_string(),
        allowed_values: None,
        input_type: None,
        added_version: "0.0.0".to_string(),
        description: "An example string property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      _number_property: ConfigProperty {
        value: 42,
        default_value: 42,
        allowed_values: None,
        input_type: None,
        added_version: "0.0.0".to_string(),
        description: "An example number property".to_string(),
        hide: true,
        deprecated_version: String::new(),
      },
      http_port: ConfigProperty {
        value: 8123,
        default_value: 8123,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Port that the http server shall listen on".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      ws_port: ConfigProperty {
        value: 8124,
        default_value: 8124,
        allowed_values: None,
        input_type: None,
        added_version: "1.0.0".to_string(),
        description: "Port that the websocket server shall listen on".to_string(),
        hide: false,
        deprecated_version: String::new(),
      },
      allow_remote_connections: ConfigProperty {
        value: true,
        default_value: true,
        allowed_values: None,
        input_type: None,
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
  #[serde(default)]
  pub user_interface_config: UserInterfaceConfig,
  pub camera_configs: Vec<CameraConfig>,
  pub open_protocol_configs: Vec<OpenProtocolClientConfig>,
  #[serde(default)]
  pub volumes: Vec<Sphere>,
  pub writer_build_info: BuildInfo
}

impl Default for Config {
  fn default() -> Self {
    Self {
      general_config: GeneralConfig::default(),
      logging_config: LoggingConfig::default(),
      user_interface_config: UserInterfaceConfig::default(),
      camera_configs: vec![CameraConfig::default()],
      open_protocol_configs: vec![OpenProtocolClientConfig::default()],
      volumes: Sphere::defaults(),
      writer_build_info: BuildInfo::default(),
    }
  }
}

/// A cartesian position in millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Position3D {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

/// A named spherical volume expressed in a given coordinate system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
  pub name: String,
  pub position: Position3D,
  pub enter_radius: f64,
  pub exit_radius: f64,
  pub coordinate_system: String,
}

impl Default for Sphere {
  fn default() -> Self {
    Self {
      name: "sphere".to_string(),
      position: Position3D::default(),
      enter_radius: 5.0,
      exit_radius: 7.5,
      coordinate_system: "world".to_string(),
    }
  }
}

impl Sphere {
  /// Example spheres, also used to generate the UI types.
  pub fn defaults() -> Vec<Sphere> {
    vec![
      Sphere {
        name: "bolt_1".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 250.0 },
        enter_radius: 5.0,
        exit_radius: 7.5,
        coordinate_system: "world".to_string(),
      },
      Sphere {
        name: "bolt_2".to_string(),
        position: Position3D { x: -100.0, y: 50.0, z: 250.0 },
        enter_radius: 5.0,
        exit_radius: 7.5,
        coordinate_system: "world".to_string(),
      },
      Sphere {
        name: "fixture".to_string(),
        position: Position3D { x: 0.0, y: 300.0, z: 0.0 },
        enter_radius: 5.0,
        exit_radius: 7.5,
        coordinate_system: "station".to_string(),
      },
    ]
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
