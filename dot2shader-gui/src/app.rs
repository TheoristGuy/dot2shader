use dot2shader::*;
use eframe::{egui, epi};

#[derive(Clone, Debug, Default)]
pub struct TemplateApp {
    pixel_art: Option<PixelArt>,
    config: DisplayConfig,
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "dot2shader"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        let Self {
            config,
            ..
        } = self;

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Configure");

            ui.separator();
            ui.label("Inline Level");
            ui.radio_value(&mut config.inline_level, InlineLevel::None, "no magic number, for Shadertoy");
            ui.radio_value(&mut config.inline_level, InlineLevel::InlineVariable, "inline constant variables, for Shadertoy");
            ui.radio_value(&mut config.inline_level, InlineLevel::Geekest, "crazy optimization, for twigl geekest");

            ui.separator();
            ui.label("Pallet Color format");
            ui.radio_value(&mut config.pallet_format, PalletFormat::IntegerDecimal, "single decimal integer");
            ui.radio_value(&mut config.pallet_format, PalletFormat::IntegerHexadecimal, "single hexadecimal integer");
            ui.radio_value(&mut config.pallet_format, PalletFormat::RGBDecimal, "vec3, specified by decimal integers");
            ui.radio_value(&mut config.pallet_format, PalletFormat::RGBHexadecimal, "vec3, specified by hexadecimal integers");
            ui.radio_value(&mut config.pallet_format, PalletFormat::RGBFloat, "vec3, specified by floats");

            ui.separator();
            ui.label("Buffer Optimization");
            ui.add(egui::Checkbox::new(&mut config.buffer_format.reverse_rows, "Turn the picture upside down."));
            ui.add(egui::Checkbox::new(&mut config.buffer_format.reverse_each_chunk, "Invert bytes of each chunk."));
            ui.add(egui::Checkbox::new(&mut config.buffer_format.force_to_raw, "Force not to compress the buffer."));
        });

        egui::CentralPanel::default().show(ctx, |ui| {


            egui::warn_if_debug_build(ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
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
