use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::fs;
use std::{env, path::PathBuf};

fn compress_wasm(bytes: &[u8]) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    std::io::Write::write_all(&mut e, bytes).unwrap();
    e.finish().unwrap()
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wasm_path = out_dir.join("compressed_wasm.bin");
    let wasm_bytes = fs::read("../wasm/pkg/cosmol_viewer_wasm_bg.wasm").unwrap();
    let compressed = compress_wasm(&wasm_bytes);
    fs::write(&wasm_path, compressed).unwrap();
}
