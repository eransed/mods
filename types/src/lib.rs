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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub ws_port: u16,
    pub log_level: String,
    pub allow_remote_connections: bool,
    pub enable_camera: bool,
    pub opencv_display: bool,
    pub skip_april_pose_estimation: bool,
    pub angle_filter: usize,
    pub min_decision_margin: f32,
    pub device_index: i32,
    pub device_width: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_port: 8123,
            ws_port: 8124,
            log_level: "info".to_string(),
            allow_remote_connections: true,
            enable_camera: true,
            opencv_display: false,
            skip_april_pose_estimation: true,
            angle_filter: 3,
            min_decision_margin: 20.0,
            device_index: 0,
            device_width: 1920 as f64,
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
}
