use base64::{Engine as _};
use cosviewer_core::{CosViewerCore, MyEguiApp};
use eframe::egui::{Vec2, ViewportBuilder};
use pyo3::{ffi::c_str, prelude::*};
use uuid::Uuid;

#[pymodule]
fn cosviewer(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CosViewer>()?;
    Ok(())
}

#[pyclass]
struct CosViewer {
    inner: CosViewerCore,
}

#[pymethods]
impl CosViewer {
    #[new]
    fn new() -> Self {
        CosViewer {
            inner: CosViewerCore::new(),
        }
    }

    fn set_state(&mut self, state: String) {
        self.inner.set_state(state);
    }

    fn get_state(&self) -> Option<String> {
        self.inner.get_state().cloned()
    }

    fn view(&self, py: Python) -> PyResult<()> {
        let is_notebook = match py.eval(c_str!("get_ipython().__class__.__name__"), None, None) {
            Ok(val) => {
                let s: &str = val.extract()?;
                s == "ZMQInteractiveShell" // Jupyter/Colab
            }
            Err(_) => false, // 没有get_ipython，默认不是Notebook
        };

        // let builtins = py.import("builtins")?;
        // let py_print = builtins.getattr("print")?;
        // py_print.call1(("这里启动Colab Notebook渲染",))?;

        if is_notebook {
            let unique_id = format!("cosviewer_{}", Uuid::new_v4());

            const JS_CODE: &str = include_str!("../../cosviewer_wasm/pkg/cosviewer_wasm.js");
            const WASM_BYTES: &[u8] =
                include_bytes!("../../cosviewer_wasm/pkg/cosviewer_wasm_bg.wasm");
            let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);
            let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);

            let html_code = format!(r#"
            <canvas id="{id}" width="300" height="150" style="width:300px; height:150px;"></canvas>
            "#, id = unique_id);

            let combined_js = format!(
            r#"
            (function() {{
                const wasmBase64 = "{wasm_base64}";
                const jsBase64 = "{js_base64}";

                // 创建 Blob 链接
                const jsCode = atob(jsBase64);
                const blob = new Blob([jsCode], {{ type: 'application/javascript' }});
                const blobUrl = URL.createObjectURL(blob);

                import(blobUrl).then(async (mod) => {{
                    const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));
                    await mod.default(wasmBytes);
                    
                    const canvas = document.getElementById('{id}');
                    const app = new mod.WebHandle();
                    await app.start(canvas);
                }});
            }})();
            "#,
                wasm_base64 = wasm_base64,
                js_base64 = js_base64,
                id = unique_id
            );

            let ipython = py.import("IPython.display")?;
            let display = ipython.getattr("display")?;

            let html = ipython.getattr("HTML")?.call1((html_code,))?;
            display.call1((html,))?;

            let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
            display.call1((js,))?;

            Ok(())
        } else {
            let native_options = eframe::NativeOptions {
                viewport: ViewportBuilder::default().with_inner_size(Vec2::new(400.0, 250.0)),
                depth_buffer: 24,
                ..Default::default()
            };
            let _ = eframe::run_native(
                "My egui App",
                native_options,
                Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
            );
            Ok(())
        }
    }
}
