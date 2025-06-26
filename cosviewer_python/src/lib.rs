use base64::{alphabet::STANDARD, engine, prelude::BASE64_STANDARD};
use cosviewer_core::{CosViewerCore, MyEguiApp};
use eframe::egui::{Vec2, ViewportBuilder};
use pyo3::{ffi::c_str, prelude::*};
use base64::{Engine as _, engine::general_purpose};

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

        let builtins = py.import("builtins")?;
        let py_print = builtins.getattr("print")?;

        if is_notebook {
            // // 打印提示
            // py_print.call1(("这里启动Colab Notebook渲染",))?;

            // // // JS 逻辑单独拆分成字符串
            // // let js_code = r#"
            // // import init, { WebHandle } from './cosviewer_wasm.js';
            // // (async () => {
            // //     // 等待 wasm 初始化
            // //     await init('./cosviewer_wasm_bg.wasm');

            // //     const canvas = document.getElementById('cosviewer');
            // //     const app = new WebHandle();
            // //     await app.start(canvas);
            // // })();
            // // "#;

            // // 静态引入 JS 和 WASM
            // const JS_CODE: &str = include_str!("../../cosviewer_wasm.js");
            // const WASM_BYTES: &[u8] = include_bytes!("../../cosviewer_wasm_bg.wasm"); 
            // let wasm_base64 = general_purpose::STANDARD.encode(WASM_BYTES);

            // let combined_js = format!(r#"
            // const wasmBase64 = "{wasm_base64}";
            // const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));

            // async function run() {{
            //     const wasmModule = await WebAssembly.instantiate(wasmBytes, {{ /* importObject 可以放这里 */ }});
            //     const init = (await eval(`{js_code}`)).default;

            //     await init(wasmModule);
            //     const canvas = document.getElementById('cosviewer');
            //     const app = new cosviewer.WebHandle();
            //     await app.start(canvas);
            // }}
            // run();
            // "#,
            // wasm_base64 = wasm_base64,
            // js_code = JS_CODE);

            // // 渲染 Canvas
            // let html_code = r#"
            // <canvas id="cosviewer" width="300" height="150" style="width:300px; height:150px; border:1px solid #FFF;"></canvas>
            // "#;


            // // 通过 IPython.display 分别显示 HTML 和执行 JS
            // let ipython = py.import("IPython.display")?;
            // let display = ipython.getattr("display")?;

            // // 显示canvas
            // let html = ipython.getattr("HTML")?.call1((html_code,))?;
            // display.call1((html,))?;

            // // 执行 JS
            // let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
            // display.call1((js,))?;

            // Ok(())


            py_print.call1(("这里启动Colab Notebook渲染",))?;

    // 1️⃣ 嵌入 JS 文件
    const JS_CODE: &str = include_str!("../../cosviewer_wasm/pkg/cosviewer_wasm.js");
    const WASM_BYTES: &[u8] = include_bytes!("../../cosviewer_wasm/pkg/cosviewer_wasm_bg.wasm");
    let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);
    let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);

    // 2️⃣ 生成 HTML
    let html_code = r#"
    <canvas id="cosviewer" width="300" height="150" style="width:300px; height:150px;"></canvas>
    "#;

    // 3️⃣ 生成 JS 启动代码
    let combined_js = format!(r#"
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
            
            const canvas = document.getElementById('cosviewer');
            const app = new mod.WebHandle();
            await app.start(canvas);
        }});
    }})();
    "#,
    wasm_base64 = wasm_base64,
    js_base64 = js_base64);

    // 4️⃣ 在 Notebook 渲染
    let ipython = py.import("IPython.display")?;
    let display = ipython.getattr("display")?;

    let html = ipython.getattr("HTML")?.call1((html_code,))?;
    display.call1((html,))?;

    let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
    display.call1((js,))?;

    Ok(())
        } else {
            println!("这里启动本地窗口渲染");
            let native_options = eframe::NativeOptions {
                viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 600.0)),
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
