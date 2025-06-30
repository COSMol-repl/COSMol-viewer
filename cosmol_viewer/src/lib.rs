use std::sync::{Arc, Mutex};

use cosmol_viewer_core::App;
pub use cosmol_viewer_core::SceneData;
pub use cosmol_viewer_core::{Scene, Sphere, utils};

use ipc_channel::ipc::{IpcReceiver, IpcSender, channel};

pub struct Viewer {
    // pub app: Arc<Mutex<Option<App>>>,
    pub sender: IpcSender<SceneData>,
}

impl Viewer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(scene: &Scene) -> Self {
        // use eframe::{egui::{Vec2, ViewportBuilder}, NativeOptions};

        // let native_options = NativeOptions {
        //     viewport: ViewportBuilder::default().with_inner_size(Vec2::new(400.0, 250.0)),
        //     depth_buffer: 24,
        //     ..Default::default()
        // };

        // let _ = eframe::run_native(
        //     "cosmol_viewer",
        //     native_options,
        //     Box::new(|cc| {
        //         use cosmol_viewer_core::AppWrapper;

        //         Ok(Box::new(AppWrapper(app.clone())))
        //     }),
        // );

//         let (tx, rx) = ipc::channel().unwrap();
// let (embedded_tx, embedded_rx) = ipc::channel().unwrap();
// // Send the IpcReceiver
// tx.send(embedded_rx).unwrap();
// // Receive the sent IpcReceiver
// let received_rx = rx.recv().unwrap();
// // Receive any data sent to the received IpcReceiver
// let rx_data = received_rx.recv().unwrap();

        use ipc_channel::ipc::{self, IpcOneShotServer};

        let (server, server_name) = IpcOneShotServer::<IpcSender<SceneData>>::new().unwrap();

        println!("Server name: {}", server_name);

        let gui_path = std::path::Path::new("cosmol_viewer_gui.exe");
        std::process::Command::new(gui_path)
            .arg(server_name)
            .spawn()
            .expect("failed to spawn gui process");
        
        let (_, sender) = server.accept().unwrap(); // 接收器传过来了！

        let guard = scene.inner.lock().unwrap();
        sender.send((*guard).clone()).unwrap();

        Viewer { sender: sender }
    }

    pub fn update(&self, scene: &Scene) {
        let guard = scene.inner.lock().unwrap();
        self.sender.send((*guard).clone()).unwrap();
    }
}
