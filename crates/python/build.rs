// use std::{env, process::Command};

fn main() {
    // let is_ci = env::var("GITHUB_ACTIONS").is_ok();

    // if is_ci {
    //     println!("cargo:warning=CI mode detected, using pre-built WASM...");
    //     return;
    // }

    // println!("cargo:warning=Building WASM in build.rs...");

    // // 在构建过程中调用 wasm-pack
    // let status = Command::new("wasm-pack")
    //     .args(["build", "../wasm", "--target", "web", "--out-dir", "../wasm/pkg"])
    //     .status()
    //     .expect("failed to run wasm-pack");

    // if !status.success() {
    //     panic!("wasm-pack build failed");
    // }
}
