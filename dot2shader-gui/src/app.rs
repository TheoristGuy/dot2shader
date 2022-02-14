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

impl Dot2ShaderApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn read_file(&self, open_dialog: bool) {
        if open_dialog {
            let message = Arc::clone(&self.message);
            let path = native_dialog::FileDialog::new()
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
                    let pallet_size_limit = pixel_art.pallet().len() <= usize::pow(2, 16);
                    if !pallet_size_limit {
                        *message.lock().unwrap() = format!(
                            "Pallet size is must be no more than {}. Pallet size: {}",
                            usize::pow(2, 16),
                            pixel_art.pallet().len()
                        );
                    }
                    pallet_size_limit
                })?;
            *message.lock().unwrap() = String::new();
            *pixel_art.lock().unwrap() = Some(new_pixel_art);
            string_update_closure()
        }
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
            .show(ctx, |ui| {
                let loaded = self.pixel_art.lock().unwrap().is_some();
                let config = &mut self.config;
                if loaded {
                    ui.heading("Configure");

                    ui.separator();
                    ui.label("Inline Level");
                    ui.radio_value(
                        &mut config.inline_level,
                        InlineLevel::None,
                        "no magic number, for Shadertoy",
                    );
                    ui.radio_value(
                        &mut config.inline_level,
                        InlineLevel::InlineVariable,
                        "inline constant variables, for Shadertoy",
                    );
                    ui.radio_value(
                        &mut config.inline_level,
                        InlineLevel::Geekest,
                        "crazy optimization, for twigl geekest",
                    );

                    let geekest = config.inline_level == InlineLevel::Geekest;
                    if geekest {
                        config.pallet_format = PalletFormat::RGBFloat;
                        config.buffer_format.force_to_raw = false;
                    }

                    ui.separator();
                    ui.label("Pallet Color format");
                    let mut add_pallet_radio = |format, string| {
                        if ui
                            .add_enabled(
                                !geekest,
                                egui::RadioButton::new(config.pallet_format == format, string),
                            )
                            .clicked()
                        {
                            config.pallet_format = format;
                        }
                    };
                    add_pallet_radio(PalletFormat::IntegerDecimal, "single decimal integer");
                    add_pallet_radio(
                        PalletFormat::IntegerHexadecimal,
                        "single hexadecimal integer",
                    );
                    add_pallet_radio(
                        PalletFormat::RGBDecimal,
                        "vec3, specified by decimal integers",
                    );
                    add_pallet_radio(
                        PalletFormat::RGBHexadecimal,
                        "vec3, specified by hexadecimal integers",
                    );
                    ui.radio_value(
                        &mut config.pallet_format,
                        PalletFormat::RGBFloat,
                        "vec3, specified by floats",
                    );

                    ui.separator();
                    ui.label("Buffer Optimization");
                    ui.add(egui::Checkbox::new(
                        &mut config.buffer_format.reverse_rows,
                        "Turn the picture upside down.",
                    ));
                    ui.add(egui::Checkbox::new(
                        &mut config.buffer_format.reverse_each_chunk,
                        "Invert bytes of each chunk.",
                    ));
                    ui.add_enabled(
                        !geekest,
                        egui::Checkbox::new(
                            &mut config.buffer_format.force_to_raw,
                            "Force not to compress the buffer.",
                        ),
                    );

                    if self.previous_config != self.config {
                        *self.message.lock().unwrap() = String::new();
                        #[cfg(not(target_arch = "wasm32"))]
                        std::thread::spawn(self.string_update_closure());
                        #[cfg(target_arch = "wasm32")]
                        {
                            let string_update_closure = self.string_update_closure();
                            wasm_bindgen_futures::spawn_local(async move {
                                string_update_closure();
                            });
                        }
                        self.previous_config = self.config;
                    }
                }
                    ui.separator();
                ui.label("");
                let open_dialog = ui.button("file open").clicked();
                self.read_file(open_dialog);
                let message = self.message.lock().unwrap().clone();
                ui.add(egui::Label::new(
                    egui::RichText::new(message).color(egui::Color32::from_rgb(255, 0, 0)),
                ));

                egui::warn_if_debug_build(ui);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.horizontal(|ui| {
                        ui.label("GitHub repo: ");
                        ui.hyperlink_to(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/eframe",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("created by ");
                        ui.hyperlink_to("@IWBTShyGuy", "https://twitter.com/IWBTShyGuy");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut string = self.string.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_sized(
                    [600.0, 100.0],
                    egui::TextEdit::multiline(&mut string).desired_rows(30),
                );
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
