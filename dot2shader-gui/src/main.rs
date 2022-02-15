#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = dot2shader_gui::Dot2ShaderApp::default();
    let icon = image::load_from_memory(include_bytes!("../resources/favicon.ico")).unwrap();
    let native_options = eframe::NativeOptions {
        icon_data: Some(eframe::epi::IconData {
            rgba: icon.to_rgba8().to_vec(),
            width: icon.width(),
            height: icon.height(),
        }),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
