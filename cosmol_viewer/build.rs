use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() {
    // println!("cargo:warning=Starting build process for GUI crate...");

    // let is_release = env::var("PROFILE").unwrap() == "release";

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
