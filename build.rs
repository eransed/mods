#![allow(unused_mut)]

use types::BuildInfo;

fn main() {
    let btu = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S%.3f %z")
        .to_string();

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_info.json");

    let build_type = if cfg!(debug_assertions) { "deb" } else { "rel" };

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

    use std::process::Command;
    // export git_hash=$(git rev-parse --short HEAD)
    let git_commit_cmd = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .expect("failed to execute process");

    // export git_branch=$(git rev-parse --abbrev-ref HEAD)
    let git_branch_cmd = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute process");

    // export git_date=$(git show -s --format=%cd --date=short HEAD)
    let git_date_cmd = Command::new("git")
        .arg("show")
        .arg("-s")
        .arg("--format=%cd")
        .arg("--date=short")
        .arg("HEAD")
        .output()
        .expect("failed to execute process");

    // export build_uname=$(uname)
    let build_uname_cmd = Command::new("uname")
        .output()
        .expect("failed to execute process");

    // export git_version=$(git --version)
    let git_version_cmd = Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    // export rustc_version=$(rustc --version)
    let rustc_version_cmd = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    // export rustc_version=$(docker --version)
    let docker_version_cmd = Command::new("docker").arg("--version").output();

    // export node_version=$(node --version)
    let node_version_cmd = Command::new("node")
        .arg("--version")
        .output()
        .expect("Failed to read node version");

    // export npm_version=$(npm --version)
    let npm_version_cmd = Command::new("npm")
        .arg("--version")
        .output()
        .expect("Failed to read npm version");

    let docker_version = match docker_version_cmd {
        Ok(v) => String::from_utf8(v.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        Err(_) => String::from("-"),
    };

    let bi = BuildInfo {
        git_hash: String::from_utf8(git_commit_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        git_branch: String::from_utf8(git_branch_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        git_date: String::from_utf8(git_date_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        git_version: String::from_utf8(git_version_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        rustc_version: String::from_utf8(rustc_version_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        docker_version,
        node_version: String::from_utf8(node_version_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        npm_version: String::from_utf8(npm_version_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
        cargo_pkg_name: env!("CARGO_PKG_NAME").to_string(),
        cargo_pkg_version: env!("CARGO_PKG_VERSION").to_string(),
        build_time_utc: btu,
        target_arch: target_arch.to_string(),
        target_avx2,
        target_neon,
        build_type: build_type.to_string(),
        build_uname: String::from_utf8(build_uname_cmd.stdout)
            .expect("Failed to convert bytes to string")
            .trim()
            .to_string(),
    };

    println!("BuildInfo: {:#?}", bi);

    let bi_json = serde_json::to_string_pretty(&bi).expect("Failed to parse json");

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("build_info.json").expect("Failed to create file");
    file.write_all(&bi_json.into_bytes())
        .expect("Failed to write file");

    let build_ui = true;

    if build_ui {
        use std::env;
        use std::path::Path;

        let root = Path::new("./ui");
        assert!(env::set_current_dir(root).is_ok());
        println!(
            "Successfully changed working directory to {}!",
            root.display()
        );

        // npm i
        let npmi = Command::new("npm")
            .arg("i")
            .output()
            .expect("failed to execute npm i");

        if !npmi.status.success() {
            panic!("npm i failed");
        }

        // npm run build
        let npmbuild = Command::new("npm")
            .arg("run")
            .arg("build")
            .output()
            .expect("failed to execute npm run build");

        if !npmbuild.status.success() {
            panic!("npm run build failed");
        }
    }

    // panic!();
}
