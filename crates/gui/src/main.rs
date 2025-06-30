use cosmol_viewer_core::{App, AppWrapper, SceneData};
use eframe::{
    NativeOptions,
    egui::{Vec2, ViewportBuilder},
};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    text: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_name = &args[1];
    let server_name = server_name.clone();

    let scene: Arc<Mutex<SceneData>> = Arc::new(Mutex::new(SceneData::new()));
    let app: Arc<Mutex<Option<App>>> = Arc::new(Mutex::new(None));

    let scene_in_thread = Arc::clone(&scene); // 这里 clone，不 move
    let app_in_thread = Arc::clone(&app);

    thread::spawn(move || {
        let tx: IpcSender<IpcSender<SceneData>> =
            IpcSender::connect(server_name.to_string()).unwrap();

        let (tx1, rx1): (IpcSender<SceneData>, IpcReceiver<SceneData>) = ipc::channel().unwrap();

        tx.send(tx1).unwrap();

        loop {
            if let Ok(scene_received) = rx1.recv() {
                let mut scene_guard = scene_in_thread.lock().unwrap();
                let mut app_guard = app_in_thread.lock().unwrap();
                if let Some(app) = &mut *app_guard {
                    println!("Received scene update");
                    app.update_scene(&scene_received);
                    app.ctx.request_repaint();
                } else {
                    println!("scene update received but app is not initialized");
                    *scene_guard = scene_received.clone();
                }
            }
        }
    });

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(400.0, 250.0)),
        depth_buffer: 24,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "cosmol_viewer",
        native_options,
        Box::new(|cc| {
            let mut guard = app.lock().unwrap();
            *guard = Some(App::new(cc, scene));
            Ok(Box::new(AppWrapper(app.clone())))
        }),
    );
}
