use std::sync::{Arc, Mutex};

/// Spawns a new thread.
#[inline]
pub fn spawn<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(closure: F) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(closure);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        closure();
    });
}

#[derive(Clone, Debug)]
pub struct FileDialogReader {
    result: Arc<Mutex<Option<Vec<u8>>>>,
    error: Arc<Mutex<Option<String>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileDialogReader {
    fn register_error(e: &impl std::fmt::Display, error: &Arc<Mutex<Option<String>>>) {
        *error.lock().unwrap() = Some(e.to_string());
    }
    /// Starts file reading
    pub fn start() -> Option<Self> {
        let result = Arc::new(Mutex::new(None));
        let error = Arc::new(Mutex::new(None));
        let path = native_dialog::FileDialog::new()
            .add_filter("pixel dot file", &["png", "bmp", "gif"])
            .show_open_single_file()
            .map_err(|e| Self::register_error(&e, &error))
            .ok()?;
        if let Some(path) = path {
            let buffer = std::fs::read(path)
                .map_err(|e| Self::register_error(&e, &error))
                .ok();
            *result.lock().unwrap() = buffer;
        }
        Some(Self { result, error })
    }
    /// Gets result of file reading. Returns `None` if the file has not been read yet.
    pub fn result(&self) -> Option<Result<Vec<u8>, String>> {
        if let Some(result) = self.result.lock().unwrap().take() {
            Some(Ok(result))
        } else {
            self.error.lock().unwrap().take().map(Err)
        }
    }
}

#[cfg(target_arch = "wasm32")]
const FILE_INPUT_NAME: &str = "file-input";
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{prelude::*, JsCast};

#[cfg(target_arch = "wasm32")]
impl FileDialogReader {
    fn get_input(error: &Arc<Mutex<Option<String>>>) -> Option<web_sys::HtmlInputElement> {
        let doc = web_sys::window().and_then(|win| win.document())?;
        let body = doc.body()?;
        let input = doc.get_element_by_id(FILE_INPUT_NAME).or_else(|| {
            (|| {
                let file_input = doc.create_element("input")?;
                file_input.set_id(FILE_INPUT_NAME);
                file_input.set_attribute("type", "file")?;
                file_input.set_attribute("style", "display:none")?;
                file_input.set_attribute("accept", "image/png, image/gif, image/bmp")?;
                body.append_child(&file_input)?;
                Ok(file_input)
            })()
            .map_err(|e: JsValue| {
                *error.lock().unwrap() =
                    Some(format!("cannot initialize file reader. JsValue: {:?}", e))
            })
            .ok()
        })?;
        Some(web_sys::HtmlInputElement::from(JsValue::from(input)))
    }
    pub fn start() -> Option<Self> {
        let result = Arc::new(Mutex::new(None));
        let error = Arc::new(Mutex::new(None));
        Self::get_input(&error)?.click();
        Some(Self { result, error })
    }
    fn start_file_read(&self, file: &web_sys::File) -> Option<()> {
        let error = Arc::clone(&self.error);
        let reader = web_sys::FileReader::new()
            .map_err(|e| {
                *error.lock().unwrap() =
                    Some(format!("cannot initialize file reader. JsValue: {:?}", e))
            })
            .ok()?;
        reader
            .read_as_array_buffer(&file)
            .map_err(|e| {
                *error.lock().unwrap() =
                    Some(format!("something wrong for read file. JsValue: {:?}", e))
            })
            .ok()?;
        let clone_reader = reader.clone();
        let clone_result = Arc::clone(&self.result);
        let closure = Closure::wrap(Box::new(move || {
            let buffer = clone_reader
                .result()
                .map(|jsvalue| js_sys::Uint8Array::new(&jsvalue).to_vec())
                .map_err(|e| {
                    *error.lock().unwrap() =
                        Some(format!("something wrong for read result. JsValue: {:?}", e));
                    e
                })?;
            *clone_result.lock().unwrap() = Some(buffer);
            Ok(())
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        reader.set_onload(Some(closure.into_js_value().unchecked_ref()));
        Some(())
    }
    pub fn result(&self) -> Option<Result<Vec<u8>, String>> {
        if let Some(result) = self.result.lock().unwrap().take() {
            return Some(Ok(result));
        } else if let Some(error) = self.error.lock().unwrap().take() {
            return Some(Err(error));
        }
        let input = Self::get_input(&self.error)?;
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            self.start_file_read(&file);
        }
        None
    }
}
