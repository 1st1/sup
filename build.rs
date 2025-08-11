use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    // Always build ripgrep from source
    let binary_path = build_ripgrep_from_source(&out_dir, &target, &host);

    // Copy binary to the sup package directory for distribution
    let binary_name = if target.contains("windows") {
        "rg.exe"
    } else {
        "rg"
    };
    let package_binary = PathBuf::from("sup/bin").join(binary_name);

    // Create bin directory if it doesn't exist
    fs::create_dir_all("sup/bin").unwrap();

    // Copy the binary to package directory
    fs::copy(&binary_path, &package_binary).expect("Failed to copy ripgrep binary to package");

    println!("Ripgrep binary copied to: {}", package_binary.display());
}

fn build_ripgrep_from_source(out_dir: &str, target: &str, host: &str) -> PathBuf {
    let version = "14.1.0";
    let ripgrep_dir = PathBuf::from(out_dir).join(format!("ripgrep-{}", version));

    // Download source if not present
    if !ripgrep_dir.exists() {
        download_ripgrep_source(&ripgrep_dir, version);
    }

    println!("Building ripgrep from source for target: {}", target);

    // Build ripgrep
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.current_dir(&ripgrep_dir);
    cargo_cmd.args(&["build", "--release"]);

    // Cross-compile if target != host
    if target != host {
        cargo_cmd.args(&["--target", target]);
    }

    let status = cargo_cmd.status().expect("Failed to build ripgrep");
    if !status.success() {
        panic!("Failed to build ripgrep from source");
    }

    // Get the built binary path
    let binary_name = if target.contains("windows") {
        "rg.exe"
    } else {
        "rg"
    };

    let built_binary = if target != host {
        ripgrep_dir
            .join("target")
            .join(target)
            .join("release")
            .join(binary_name)
    } else {
        ripgrep_dir.join("target").join("release").join(binary_name)
    };

    let dest_binary = PathBuf::from(out_dir).join(binary_name);

    fs::copy(&built_binary, &dest_binary).expect("Failed to copy ripgrep binary");

    println!(
        "Ripgrep binary built successfully at: {}",
        dest_binary.display()
    );

    dest_binary
}

fn download_ripgrep_source(dest_dir: &Path, version: &str) {
    let url = format!(
        "https://github.com/BurntSushi/ripgrep/archive/refs/tags/{}.tar.gz",
        version
    );

    println!("Downloading ripgrep source from: {}", url);

    let tar_path = dest_dir.with_extension("tar.gz");
    let parent_dir = dest_dir.parent().unwrap();
    fs::create_dir_all(parent_dir).unwrap();

    // Download using git clone instead of curl for better reliability
    // First try git clone
    let git_url = format!("https://github.com/BurntSushi/ripgrep.git");
    let status = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "--branch",
            version,
            &git_url,
            dest_dir.to_str().unwrap(),
        ])
        .status();

    if status.is_ok() && status.unwrap().success() {
        println!("Successfully cloned ripgrep source using git");
        return;
    }

    // Fallback to curl if git clone fails
    println!("Git clone failed, falling back to curl...");

    let status = Command::new("curl")
        .args(&["-L", "-o", tar_path.to_str().unwrap(), &url])
        .status()
        .expect("Failed to download ripgrep source");

    if !status.success() {
        panic!("Failed to download ripgrep source from {}", url);
    }

    // Extract
    let status = Command::new("tar")
        .args(&[
            "-xzf",
            tar_path.to_str().unwrap(),
            "-C",
            parent_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to extract ripgrep source");

    if !status.success() {
        panic!("Failed to extract ripgrep source");
    }

    // Clean up
    fs::remove_file(tar_path).ok();

    println!("Successfully downloaded and extracted ripgrep source");
}
