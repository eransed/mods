#![allow(unused_mut)]
use chrono::{DateTime, Local};
#[cfg(feature = "sensor")]
use semver::{Version, VersionReq};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use types::{BuildInfo, Config};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

fn ts() -> String {
  let local: DateTime<Local> = Local::now();
  format!("{}", local.format("%Y-%m-%d %H:%M:%S%.3f"))
}

fn _cross_command(cmd: &str) -> Command {
  p!("{} cross_command: {}", ts(), cmd);
  if cfg!(windows) {
    let shell = env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string());
    let mut command = Command::new(shell);
    command.arg("/C").arg(cmd);
    command
  } else {
    Command::new(cmd)
  }
}

#[macro_export]
macro_rules! cross_command {
    ($cmd:expr $(, $arg:expr )* $(,)?) => {{

        let ts = ts();
        use std::time::Instant;
        let now = Instant::now();

        let mut args = Vec::<String>::new();
        $(
            args.push($arg.to_string());
        )*

        let quiet = true;

        #[cfg(windows)]
        {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg($cmd);
            cmd.args(&args);
            let out = cmd.output();
            if !quiet {
                println!("cargo::warning={}", format!("{} cmd /C {} {} [{:.1?}]", ts, $cmd, args.join(" "), now.elapsed()));
            }
            out
        }

        #[cfg(not(windows))]
        {
            let mut cmd = Command::new($cmd);
            cmd.args(&args);
            let out = cmd.output();
            if !quiet {
                println!("cargo::warning={}", format!("{} {} {} [{:.1?}]", ts, $cmd, args.join(" "), now.elapsed()));
            }
            out
        }
    }};
}

fn main() {
  let simple_compile = false;

  let ocv_ver_str;

  #[cfg(not(feature = "sensor"))]
  {
    ocv_ver_str = String::from("Compiled without opencv");
  }

  #[cfg(feature = "sensor")]
  {
    let opencv_version = cross_command!("opencv_version").expect("Failed to read opencv_version");
    println!("cargo::rustc-check-cfg=cfg(opencv_pre_411)");
    println!("cargo::rustc-check-cfg=cfg(opencv4)");
    println!("cargo::rustc-check-cfg=cfg(opencv5)");
    let opencv_req = VersionReq::parse("<=4.10.0").expect("Failed to parse opencv_version required version");

    ocv_ver_str = String::from_utf8(opencv_version.stdout)
      .expect("Failed to convert bytes ocv_ver_str to string")
      .trim()
      .to_string();

    let opencv_ver = Version::parse(&ocv_ver_str).expect("Failed to parse opencv_version version");
    let opencv_pre_411 = opencv_req.matches(&opencv_ver);
    if ocv_ver_str.starts_with("4.") {
      println!("cargo::rustc-cfg=opencv4");
      if opencv_pre_411 {
        println!("cargo::rustc-cfg=opencv_pre_411");
      }
    } else if ocv_ver_str.starts_with("5.") {
      println!("cargo::rustc-cfg=opencv5");
    }
  }

  println!("cargo::rustc-link-search=native=/usr/local/lib");
  println!("cargo::rustc-link-arg=-Wl,-rpath,/usr/local/lib");

  if simple_compile {
    return;
  }

  let btu = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %z").to_string();

  let _ = cross_command!("echo Start");

  // will cause recompilation every time as build.rs modifies them:
  let build_ui = true;
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=ui");
  println!("cargo::rerun-if-changed=build_info.json");

  let build_type = if cfg!(debug_assertions) { "debug" } else { "release" };

  let mut target_arch;
  let mut target_avx2 = false;
  let mut target_neon = false;
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  {
    use std::arch::is_x86_feature_detected;
    if is_x86_feature_detected!("avx2") {
      target_avx2 = true;
    }
  }

  #[cfg(target_arch = "x86")]
  {
    target_arch = "x86";
  }

  #[cfg(target_arch = "x86_64")]
  {
    target_arch = "x86_64";
  }

  #[cfg(target_arch = "arm")]
  {
    target_arch = "arm";
  }

  #[cfg(target_arch = "aarch64")]
  {
    use std::arch::is_aarch64_feature_detected;
    target_arch = "aarch64";
    if is_aarch64_feature_detected!("neon") {
      target_neon = true;
    }
  }

  let git_commit_cmd =
    cross_command!("git", "rev-parse", "--short", "HEAD").expect("Failed to execute git_commit_cmd process");

  let git_branch_cmd =
    cross_command!("git", "rev-parse", "--abbrev-ref", "HEAD").expect("Failed to execute git_branch_cmd process");

  let git_date_cmd = cross_command!("git", "show", "-s", "--format=%cd", "--date=short", "HEAD")
    .expect("Failed to execute git_date_cmd process");

  let build_uname_cmd = cross_command!("uname").expect("Failed to execute build_uname_cmd process");

  let git_version_cmd = cross_command!("git", "--version").expect("Failed to execute git_version_cmd process");

  let rustc_version_cmd = cross_command!("rustc", "--version").expect("Failed to execute rustc_version_cmd process");

  let docker_version_cmd = cross_command!("docker", "--version");

  let node_version_cmd = cross_command!("node", "--version").expect("Failed to read node version");

  let npm_version_cmd = cross_command!("npm", "--version").expect("Failed to read npm version");

  let quicktype_version_cmd =
    cross_command!("quicktype", "--version").expect("Failed to read quicktype version");

  let docker_version = match docker_version_cmd {
    Ok(v) => {
      String::from_utf8(v.stdout).expect("Failed to convert docker_version bytes to string").trim().to_string()
    }
    Err(_) => String::from("-"),
  };

  let du_release_mods_size_kb_cmd =
    cross_command!("du", "-k", "target/release/mods").expect("Failed to read release mods size");

  let du_debug_mods_size_kb_cmd =
    cross_command!("du", "-k", "target/debug/mods").expect("Failed to read debug mods size");

  let du_html_kb_cmd = cross_command!("du", "-k", "ui/dist/index.html")
    .expect("Failed to read ui/dist/index.html size");

  let du_js_kb_cmd =
    cross_command!("du", "-k", "ui/dist/main.js").expect("Failed to read ui/dist/main.js size");

  let du_css_kb_cmd =
    cross_command!("du", "-k", "ui/dist/main.css").expect("Failed to read ui/dist/main.css size");

  let bi = BuildInfo {
    compiled_with_sensor_support: cfg!(feature = "sensor"),
    binary_release_size_kb: du_release_mods_size_kb_cmd
      .stdout
      .split(|&b| b == b'\t')
      .next()
      .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default())
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or_default(),
    binary_debug_size_kb: du_debug_mods_size_kb_cmd
      .stdout
      .split(|&b| b == b'\t')
      .next()
      .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default())
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or_default(),
    index_html_size_kb: du_html_kb_cmd
      .stdout
      .split(|&b| b == b'\t')
      .next()
      .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default())
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or_default(),
    main_js_size_kb: du_js_kb_cmd
      .stdout
      .split(|&b| b == b'\t')
      .next()
      .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default())
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or_default(),
    main_css_size_kb: du_css_kb_cmd
      .stdout
      .split(|&b| b == b'\t')
      .next()
      .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default())
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or_default(),
    git_hash: String::from_utf8(git_commit_cmd.stdout)
      .expect("Failed to convert git_hash bytes to string")
      .trim()
      .to_string(),
    git_branch: String::from_utf8(git_branch_cmd.stdout)
      .expect("Failed to convert git_branch bytes to string")
      .trim()
      .to_string(),
    git_date: String::from_utf8(git_date_cmd.stdout)
      .expect("Failed to convert git_date bytes to string")
      .trim()
      .to_string(),
    git_version: String::from_utf8(git_version_cmd.stdout)
      .expect("Failed to convert git_version bytes to string")
      .trim()
      .to_string(),
    rustc_version: String::from_utf8(rustc_version_cmd.stdout)
      .expect("Failed to convert rustc_version bytes to string")
      .trim()
      .to_string(),
    docker_version,
    node_version: String::from_utf8(node_version_cmd.stdout)
      .expect("Failed to convert node_version bytes to string")
      .trim()
      .to_string(),
    npm_version: String::from_utf8(npm_version_cmd.stdout)
      .expect("Failed to convert npm_version bytes to string")
      .trim()
      .to_string(),
    quicktype_version: String::from_utf8(quicktype_version_cmd.stdout)
      .expect("Failed to convert quicktype_version bytes to string")
      .trim()
      .to_string(),
    opencv_version: ocv_ver_str,
    cargo_pkg_name: env!("CARGO_PKG_NAME").to_string(),
    cargo_pkg_version: env!("CARGO_PKG_VERSION").to_string(),
    build_time_utc: btu,
    target_arch: target_arch.to_string(),
    target_avx2,
    target_neon,
    build_type: build_type.to_string(),
    build_uname: String::from_utf8(build_uname_cmd.stdout)
      .expect("Failed to convert build_uname bytes to string")
      .trim()
      .to_string(),
    windows: cfg!(windows),
  };

  let bi_json = serde_json::to_string_pretty(&bi).expect("Failed to parse build_info json");

  let mut file = File::create("build_info.json").expect("Failed to create build_info file");
  file.write_all(&bi_json.into_bytes()).expect("Failed to write build_info.json file");

  let quicktype_build_info_cmd = cross_command!(
    "quicktype",
    "--lang",
    "ts",
    "--just-types",
    "build_info.json",
    "--out",
    "ui/src/types/BuildInfo.ts"
  )
  .expect("Failed to convert build_info.json to BuildInfo.ts: Command creation failed");

  if !quicktype_build_info_cmd.status.success() {
    panic!("Failed to convert build_info.json to BuildInfo.ts: Command failed");
  }

  let config = Config::default();
  let config_serialized = serde_json::to_string(&config).expect("Failed to serialize config to json");
  let mut default_config_file = File::create("default_config.json").expect("Failed to create default_config file");
  default_config_file.write_all(&config_serialized.into_bytes()).expect("Failed to write default_config file");

  let quicktype_config_cmd = cross_command!(
    "quicktype",
    "--lang",
    "ts",
    "--just-types",
    "default_config.json",
    "--out",
    "ui/src/types/Config.ts"
  ).expect("Failed to convert default_config.json to Config.ts: Command creation failed");

    if !quicktype_config_cmd.status.success() {
    panic!("Failed to convert default_config.json to Config.ts: Command failed");
  }

  if build_ui {
    use std::env;
    use std::path::Path;

    let root = Path::new("./ui");
    assert!(env::set_current_dir(root).is_ok());
    println!("Successfully changed working directory to {}!", root.display());

    let npmi = cross_command!("npm", "i").expect("Failed to execute npm i");

    if !npmi.status.success() {
      panic!("npm i failed");
    }

    let npmbuild = cross_command!("npm", "run", "build").expect("Failed to execute npm run build");

    if !npmbuild.status.success() {
      panic!("npm run build failed");
    }
  }

  let _ = cross_command!("echo", "Done");
}
