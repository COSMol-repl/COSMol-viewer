use std::process::Command;
use std::env;

fn main() {
    println!("cargo:warning=Building WASM in build.rs...");

    // 在构建过程中调用 wasm-pack
    let status = Command::new("wasm-pack")
        .args(["build", "../cosviewer_wasm", "--target", "web"])
        .status()
        .expect("failed to run wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }
}
