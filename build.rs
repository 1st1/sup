use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ripgrep-src");

    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    // Build ripgrep from submodule
    let binary_path = build_ripgrep_from_submodule(&out_dir, &target, &host);

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

fn build_ripgrep_from_submodule(out_dir: &str, target: &str, host: &str) -> PathBuf {
    // Use the submodule
    let ripgrep_dir = PathBuf::from("ripgrep-src");

    if !ripgrep_dir.exists() {
        panic!("ripgrep-src submodule not found! Run: git submodule update --init --recursive");
    }

    println!("Building ripgrep from submodule for target: {}", target);

    // Build ripgrep
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.current_dir(&ripgrep_dir);
    cargo_cmd.args(&["build", "--release", "--bin", "rg"]);

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