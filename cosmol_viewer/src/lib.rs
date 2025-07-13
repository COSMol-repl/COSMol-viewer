use std::env;
use std::fs::File;
use std::io::Write;
use cosmol_viewer_core::scene::Scene;
pub use cosmol_viewer_core::{utils, scene, parser};
use ipc_channel::ipc::IpcOneShotServer;

use ipc_channel::ipc::IpcSender;
use sha2::{Digest, Sha256};

pub struct Viewer {
    // pub app: Arc<Mutex<Option<App>>>,
    pub sender: IpcSender<Scene>,
}

#[cfg(all(debug_assertions, target_os = "windows"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../target/debug/cosmol_viewer_gui.exe");

#[cfg(all(debug_assertions, not(target_os = "windows")))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../target/debug/cosmol_viewer_gui");

#[cfg(all(not(debug_assertions), target_os = "windows"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../target/release/cosmol_viewer_gui.exe");

#[cfg(all(not(debug_assertions), not(target_os = "windows")))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../target/release/cosmol_viewer_gui");

fn calculate_gui_hash() -> String {
    let result = Sha256::digest(GUI_EXE_BYTES);
    hex::encode(result)
}

fn cleanup_old_temp_gui_files() -> std::io::Result<()> {
    let tmp_dir = env::temp_dir();

    for entry in std::fs::read_dir(&tmp_dir)? {
        let path = entry?.path();
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if file_name.starts_with("cosmol_temp_gui_") && file_name.ends_with(".exe") {
                let _ = std::fs::remove_file(&path); // 忽略失败（比如有进程在用）
            }
        }
    }

    Ok(())
}

fn extract_and_run_gui(arg: &str) -> std::io::Result<()> {
    cleanup_old_temp_gui_files()?;
    
    let tmp_dir = env::temp_dir();
    let exe_path = tmp_dir.join(format!("cosmol_temp_gui_{}.exe", calculate_gui_hash()));

    if !exe_path.exists() {
        let mut file = File::create(&exe_path)?;
        file.write_all(GUI_EXE_BYTES)?;
    }

    println!("Launching GUI from: {}", exe_path.display());

    std::process::Command::new(&exe_path)
        .arg(arg)
        .spawn()
        .expect("Failed to launch GUI process");

    Ok(())
}

impl Viewer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(scene: &Scene) -> Self {
        let (server, server_name) = IpcOneShotServer::<IpcSender<Scene>>::new().unwrap();

        extract_and_run_gui(&server_name).expect("Failed to extract and run GUI executable");

        let (_, sender) = server.accept().unwrap(); 
        sender.send(scene.clone()).unwrap();

        // panic!("scene: {}", serde_json::to_string(&scene).unwrap());

        Viewer { sender: sender }
    }

    pub fn update(&self, scene: &Scene) {
        self.sender.send(scene.clone()).unwrap();
    }
}
