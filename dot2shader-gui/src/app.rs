use dot2shader::*;
use eframe::{egui, epi};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct Dot2ShaderApp {
    pixel_art: Arc<Mutex<Option<PixelArt>>>,
    string: Arc<Mutex<String>>,
    message: Arc<Mutex<String>>,
    config: DisplayConfig,
    previous_config: DisplayConfig,
}

/// file io
impl Dot2ShaderApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn read_file(&self, open_dialog: bool) {
        if open_dialog {
            let message = Arc::clone(&self.message);
            let path = native_dialog::FileDialog::new()
                .add_filter("pixel dot files", &["png", "bmp", "gif"])
                .show_open_single_file()
                .unwrap_or_else(|e| {
                    *message.lock().unwrap() = e.to_string();
                    None
                });
            let pixel_art_update_closure = self.pixel_art_update_closure();
            std::thread::spawn(move || {
                pixel_art_update_closure(path.and_then(|path| {
                    std::fs::read(path)
                        .map_err(|e| {
                            *message.lock().unwrap() = e.to_string();
                        })
                        .ok()
                }));
            });
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn read_file(&self, open_dialog: bool) -> Option<()> {
        use eframe::wasm_bindgen::{prelude::*, JsCast};
        let doc = web_sys::window()
            .and_then(|win| win.document())
            .expect("failed to init document");
        let input =
            web_sys::HtmlInputElement::from(JsValue::from(doc.get_element_by_id("file-input")));
        if open_dialog {
            input.click();
        }
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            web_sys::console::log_1(&JsValue::from(&file.name()));
            let message = Arc::clone(&self.message);
            let reader = web_sys::FileReader::new()
                .map_err(|e| {
                    *message.lock().unwrap() =
                        format!("cannot initialize file reader. JsValue: {:?}", e)
                })
                .ok()?;
            reader
                .read_as_array_buffer(&file)
                .map_err(|e| {
                    *message.lock().unwrap() =
                        format!("something wrong for read file. JsValue: {:?}", e)
                })
                .ok()?;
            let clone_reader = reader.clone();
            let clone_message = Arc::clone(&message);
            let pixel_art_update_closure = self.pixel_art_update_closure();
            let closure = Closure::wrap(Box::new(move || {
                pixel_art_update_closure(
                    clone_reader
                        .result()
                        .map(|jsvalue| js_sys::Uint8Array::new(&jsvalue).to_vec())
                        .map_err(|e| {
                            *clone_message.lock().unwrap() =
                                format!("something wrong for read result. JsValue: {:?}", e);
                            e
                        })
                        .ok(),
                );
                Ok(())
            }) as Box<dyn FnMut() -> Result<(), JsValue>>);
            reader.set_onload(Some(closure.into_js_value().unchecked_ref()));
        }
        Some(())
    }
    fn string_update_closure(&self) -> impl Fn() -> Option<()> + 'static {
        let pixel_art = Arc::clone(&self.pixel_art);
        let string = Arc::clone(&self.string);
        let message = Arc::clone(&self.message);
        let config = self.config;
        move || {
            let pixel_art = pixel_art.lock().unwrap().clone()?;
            let display = pixel_art
                .display(config)
                .map_err(|e| *message.lock().unwrap() = e.to_string())
                .ok()?;
            let new_string = display.to_string();
            *string.lock().unwrap() = new_string;
            Some(())
        }
    }
    fn pixel_art_update_closure(&self) -> impl Fn(Option<Vec<u8>>) -> Option<()> + 'static {
        let message = Arc::clone(&self.message);
        let pixel_art = Arc::clone(&self.pixel_art);
        let string_update_closure = self.string_update_closure();
        move |buffer| {
            let new_pixel_art = buffer
                .filter(|buffer| {
                    let short = buffer.len() < 1024 * 15;
                    if !short {
                        *message.lock().unwrap() = format!(
                            "File size must be less than 15KB. file size: {}KB",
                            buffer.len() / 1024
                        );
                    }
                    short
                })
                .and_then(|buffer| {
                    PixelArt::from_image(&buffer)
                        .map_err(|e| *message.lock().unwrap() = e.to_string())
                        .ok()
                })
                .filter(|pixel_art| {
                    let palette_size_limit = pixel_art.palette().len() <= usize::pow(2, 16);
                    if !palette_size_limit {
                        *message.lock().unwrap() = format!(
                            "Palette size is must be no more than {}. Palette size: {}",
                            usize::pow(2, 16),
                            pixel_art.palette().len()
                        );
                    }
                    palette_size_limit
                })?;
            *message.lock().unwrap() = String::new();
            *pixel_art.lock().unwrap() = Some(new_pixel_art);
            string_update_closure()
        }
    }
}

/// panel setting
impl Dot2ShaderApp {
    fn is_geekest_mode(&self) -> bool {
        self.config.inline_level == InlineLevel::Geekest
    }
    fn inline_level_setting(&mut self, ui: &mut egui::Ui) {
        use InlineLevel::*;
        let inline_level = &mut self.config.inline_level;
        ui.label("Inline Level");
        let mut set_radio_value = move |val, msg| ui.radio_value(inline_level, val, msg);
        set_radio_value(None, "no magic number, for Shadertoy");
        set_radio_value(InlineVariable, "inline constant variables, for Shadertoy");
        set_radio_value(Geekest, "crazy optimization, for twigl geekest");
    }
    fn pallet_color_format_setting(&mut self, ui: &mut egui::Ui) {
        use PaletteFormat::*;
        let geekest = self.is_geekest_mode();
        let palette_format = &mut self.config.palette_format;
        if geekest {
            *palette_format = RGBFloat;
        }
        ui.label("Palette Color Format");
        let mut add_palette_radio = |format, string| {
            let button = egui::RadioButton::new(*palette_format == format, string);
            if ui.add_enabled(!geekest, button).clicked() {
                *palette_format = format;
            }
        };
        add_palette_radio(IntegerDecimal, "single decimal integer");
        add_palette_radio(IntegerHexadecimal, "single hexadecimal integer");
        add_palette_radio(RGBDecimal, "vec3, specified by decimal integers");
        add_palette_radio(RGBHexadecimal, "vec3, specified by hexadecimal integers");
        ui.radio_value(palette_format, RGBFloat, "vec3, specified by floats");
    }
    fn buffer_format_setting(&mut self, ui: &mut egui::Ui) {
        let geekest = self.is_geekest_mode();
        let buffer_format = &mut self.config.buffer_format;
        if geekest {
            buffer_format.force_to_raw = false;
        }
        ui.label("Buffer Optimization");
        ui.checkbox(
            &mut buffer_format.reverse_rows,
            "Turn the picture upside down.",
        );
        ui.checkbox(
            &mut buffer_format.reverse_each_chunk,
            "Invert bytes of each chunk.",
        );
        let check_force_to_raw = egui::Checkbox::new(
            &mut buffer_format.force_to_raw,
            "Force not to compress the buffer.",
        );
        ui.add_enabled(!geekest, check_force_to_raw);
    }
    fn setting_change_string_update(&mut self) {
        if self.previous_config != self.config {
            *self.message.lock().unwrap() = String::new();
            let string_update_closure = self.string_update_closure();
            #[cfg(not(target_arch = "wasm32"))]
            std::thread::spawn(string_update_closure);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(async move {
                string_update_closure();
            });
            self.previous_config = self.config;
        }
    }
    fn file_open_button(&self, ui: &mut egui::Ui) {
        self.read_file(ui.button("File Open...").clicked());
    }
    fn copy_button(&self, ui: &mut egui::Ui) {
        if ui.button("Copy Code").clicked() {
            ui.output().copied_text = self.string.lock().unwrap().clone();
        }
    }
    fn error_message_label(&mut self, ui: &mut egui::Ui) {
        let message = self.message.lock().unwrap().clone();
        ui.add(egui::Label::new(
            egui::RichText::new(message).color(egui::Color32::from_rgb(255, 0, 0)),
        ));
    }
    fn bottom_credit(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                ui.label("GitHub repo: coming coon...");
            });
            ui.horizontal(|ui| {
                ui.label("created by ");
                ui.hyperlink_to("@IWBTShyGuy", "https://twitter.com/IWBTShyGuy");
            });
        });
    }
    fn side_panel_rayout(&mut self, ui: &mut egui::Ui) {
        let loaded = self.pixel_art.lock().unwrap().is_some();
        if loaded {
            ui.heading("Configure");
            ui.separator();
            self.inline_level_setting(ui);
            ui.separator();
            self.pallet_color_format_setting(ui);
            ui.separator();
            self.buffer_format_setting(ui);
            self.setting_change_string_update();
        }
        ui.separator();
        ui.label("");
        if loaded {
            ui.horizontal(|ui| {
                self.copy_button(ui);
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    self.file_open_button(ui);
                })
            });
        } else {
            self.file_open_button(ui);
        }
        self.error_message_label(ui);
        egui::warn_if_debug_build(ui);
        self.bottom_credit(ui);
    }
}

impl epi::App for Dot2ShaderApp {
    fn name(&self) -> &str {
        "dot2shader"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        frame.set_window_size([1600.0, 1200.0].into());
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        ctx.set_pixels_per_point(4.0 / 3.0);
        egui::SidePanel::left("side_panel")
            .default_width(290.0)
            .resizable(false)
            .show(ctx, |ui| self.side_panel_rayout(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut string = self.string.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_sized(
                    [600.0, 100.0],
                    egui::TextEdit::multiline(&mut string).desired_rows(30),
                );
            });
        });
    }
}
