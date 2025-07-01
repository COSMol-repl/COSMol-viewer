use std::{env, process::Command};

fn main() {
    let is_ci = env::var("GITHUB_ACTIONS").is_ok();

    if is_ci {
        println!("cargo:warning=CI mode detected, using pre-built WASM...");
        return;
    }

    println!("cargo:warning=Building WASM in build.rs...");

    // 在构建过程中调用 wasm-pack
    let status = Command::new("wasm-pack")
        .args(["build", "../wasm", "--target", "web", "--out-dir", "../wasm/pkg"])
        .status()
        .expect("failed to run wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }

    // println!("cargo:warning=Starting build process for GUI crate...");

    // let is_release = env::var("PROFILE").unwrap() == "release";

    // println!("cargo:warning=Release mode: {}", is_release);

    // // 构建 GUI 子 crate
    // println!("cargo:warning=Building GUI crate...");
    // let status = Command::new("cargo")
    //     .arg("build")
    //     .arg("--package")
    //     .arg("cosmol_viewer_gui")
    //     .args(if is_release { vec!["--release"] } else { vec![] })
    //     .status()  // 👈 加上这个！
    //     .expect("Failed to build GUI crate");

    // if !status.success() {
    //     panic!("Failed to compile GUI executable");
    // }
}
