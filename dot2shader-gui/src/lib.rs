#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::Dot2ShaderApp;

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), eframe::wasm_bindgen::JsValue> {
    let doc = web_sys::window()
        .and_then(|win| win.document())
        .expect("failed to init document");
    let body = doc.body().expect("failed to get body");
    let canvas = doc.create_element("canvas")?;
    canvas.set_id("canvas");
    body.append_child(&canvas)?;
    let file_input = doc.create_element("input")?;
    file_input.set_id("file-input");
    file_input.set_attribute("type", "file")?;
    file_input.set_attribute("style", "display:none")?;
    body.append_child(&file_input)?;

    let app = Dot2ShaderApp::default();
    eframe::start_web("canvas", Box::new(app))
}
