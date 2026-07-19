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
    pub target_arch: String,
    pub target_avx2: bool,
    pub target_neon: bool,
    pub windows: bool,
}