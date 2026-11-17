use egui_macroquad::egui;
use crate::GUI_WINDOW_WIDTH;
use crate::render::{RenderOptions, RenderType};
use crate::simulation::{SimulationSnapshot, SimulationWorker};


pub struct Gui {
    pub initialized: bool,
    pub is_stopped: bool,
    pub interval_ms: u32,
    pub light_energy: f32,
    pub ticks_per_season: u32,
    pub corpse_ratio: f32,
    pub render_options: RenderOptions
}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            initialized: false,
            is_stopped: true,
            interval_ms: 10,
            light_energy: 2.0,
            ticks_per_season: 0,
            corpse_ratio: 0.5,
            render_options: RenderOptions::default()
        }
    }
}

impl Gui {
    pub fn draw_gui(
        &mut self,
        ctx: &egui::Context,
        snapshot: &SimulationSnapshot,
        simulation: &mut SimulationWorker) {
        if !self.initialized {
            ctx.set_pixels_per_point(2.0);

            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::WHITE);
            ctx.set_visuals(visuals);

            self.initialized = true;
        }

        egui::SidePanel::right("control_panel")
            .exact_width(GUI_WINDOW_WIDTH)
            .resizable(false)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label("Main");
                    ui.label(format!("steps: {}", snapshot.ticks));
                    ui.label(format!("(interval {}, speed {} ticks/sec)", self.interval_ms, snapshot.actual_ticks_per_second));
                    ui.label(format!("Object total: {}", snapshot.object_count()));
                    ui.label(format!("Season: {}", snapshot.season));
                });

                self.draw_control_window(ui, simulation);

                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label("Render");
                    ui.radio_value(&mut self.render_options.mode, RenderType::Natural, "Natural");
                    ui.radio_value(&mut self.render_options.mode, RenderType::Predators, "Predators");
                    ui.radio_value(&mut self.render_options.mode, RenderType::Energy, "Energy");
                    ui.radio_value(&mut self.render_options.mode, RenderType::Reproduction, "Reproduction");
                });
            });
    }

    fn draw_control_window(&mut self, ui: &mut egui::Ui, simulation: &mut SimulationWorker) {
        ui.group(|ui| {
            let size = ui.available_width();
            ui.set_width(size);
            ui.label("Controls");
            if ui.add_sized([size / 2.0, 0.0], egui::Button::new(if self.is_stopped { "Start" } else { "Stop" })).clicked() {
                self.is_stopped = !self.is_stopped;
                simulation.set_paused(self.is_stopped);
            }
            if ui.add_sized([size, 0.0], egui::Slider::new(&mut self.interval_ms, 100..=0)
                .text("Interval (ms)"))
                .changed() {
                simulation.set_interval(self.interval_ms);
            }

            if ui.add_sized([size, 0.0], egui::Slider::new(&mut self.light_energy, 0.0..=5.0)
                .step_by(0.5)
                .text("Photo energy"))
                .changed() {
                simulation.set_photosynthesis_multiplier(self.light_energy);
            }

            if ui.add_sized([size, 0.0], egui::Slider::new(&mut self.ticks_per_season, 0..=5000)
                .step_by(250.0)
                .text("Ticks/Season"))
                .changed() {
                simulation.set_ticks_per_season(self.ticks_per_season);
            }

            if ui.add_sized([size, 0.0], egui::Slider::new(&mut self.corpse_ratio, 0.0..=1.0)
                .step_by(0.05)
                .text("Corpse Energy %"))
                .changed() {
                simulation.set_corpse_energy_ratio(self.corpse_ratio);
            }
        });
    }
}