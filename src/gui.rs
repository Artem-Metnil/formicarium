use std::collections::VecDeque;
use egui_macroquad::egui;
use egui_macroquad::egui::Context;
use egui_plot::{Line, Plot, PlotPoints};
use crate::ant::Ant;
use crate::graph_brain::Brain;
use crate::GUI_WINDOW_WIDTH;
use crate::render::{RenderOptions, RenderType};
use crate::simulation::{SimulationSnapshot, SimulationWorker};

pub struct DnaFile {
    pub name: String,
    pub brain: Brain
}

pub struct Gui {
    pub initialized: bool,
    pub is_stopped: bool,
    pub interval_ms: u32,
    pub light_energy: f32,
    pub ticks_per_season: u32,
    pub corpse_ratio: f32,
    pub render_options: RenderOptions,

    show_chart: bool,
    history: VecDeque<[f64; 2]>,
    last_sampled_tick: u64,
    sample_step: u64,

    pub selected_ant: Option<Ant>,

    show_dna_library: bool,
    pub dna_library: Vec<DnaFile>,
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
            render_options: RenderOptions::default(),

            show_chart: false,
            history: VecDeque::with_capacity(2048),
            last_sampled_tick: 0,
            sample_step: 50,
            selected_ant: None,

            show_dna_library: false,
            dna_library: load_dna_files(),
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

                ui.group(|ui| {
                    let size = ui.available_width();
                   ui.set_width(size);
                    ui.label("Additional Windows");
                    if ui.add_sized([size / 2.0, 0.0], egui::Button::new("Chart")).clicked() {
                        self.show_chart = true;
                    }
                });
            });

        self.draw_population_chart(ctx, snapshot);
        self.draw_ant_inspector(ctx);
    }

    fn draw_population_chart(&mut self, ctx: &Context, snapshot: &SimulationSnapshot) {
        if self.show_chart {
            let mut is_open = self.show_chart;

            if snapshot.ticks < self.last_sampled_tick {
                self.history.clear();
                self.last_sampled_tick = snapshot.ticks;
            }

            if self.history.is_empty() || snapshot.ticks.saturating_sub(self.last_sampled_tick) >= self.sample_step {
                if self.history.len() >= 2048 {
                    self.history.pop_front();
                }
                self.history.push_back([snapshot.ticks as f64, snapshot.object_count() as f64]);
                self.last_sampled_tick = snapshot.ticks;
            }

            egui::Window::new("Population chart")
                .open(&mut is_open)
                .default_width(720.0)
                .default_height(480.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Precision: ");
                        ui.add(egui::Slider::new(&mut self.sample_step, 1..=1000)
                            .logarithmic(true)
                            .suffix(" ticks"));
                    });

                    ui.separator();
                    let points: PlotPoints = self.history.iter().copied().collect();

                    Plot::new("pop_plot")
                        .legend(egui_plot::Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points).name("Total Ants"))
                        });
                });

            if !is_open {
                self.show_chart = false;
                self.history.clear();
                self.last_sampled_tick = 0;
            }
        }
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

    fn draw_ant_inspector(&mut self, ctx: &Context) {
        if let Some(ant) = &self.selected_ant {
            let mut is_open = true;

            egui::Window::new("Ant Inspector")
                .open(&mut is_open)
                .default_pos([20.0, 20.0])
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.label(format!("Pos: ({}, {})", ant.x, ant.y));
                    ui.label(format!("Energy: {:.1}", ant.energy));
                    ui.label(format!("Age: {}", ant.age));
                    ui.label(format!("Tag: {:#X}", ant.get_tag()));
                    ui.label(format!("Metabolic: {}", ant.get_brain_copy().metabolic_cost()));

                    ui.separator();
                    ui.label("=== Diet Scores ===");
                    ui.label(format!("Photo: {:.1}", ant.score_photosynthesis));
                    ui.label(format!("Pred:  {:.1}", ant.score_predators));
                    ui.label(format!("Herb:  {:.1}", ant.score_herbivorous));

                    ui.separator();
                    ui.columns(2, |columns| {
                        if columns[0].button("Show brain").clicked() {

                        }
                        if columns[1].button("Save DNA").clicked() {
                            std::fs::create_dir_all("saved_dna").ok();
                            let filename = format!("saved_dna/dna_{:X}.dna", ant.get_tag());
                            let bytes = ant.get_brain_copy().to_bytes();
                            let result = std::fs::write(filename, bytes);
                            match result {
                                Ok(_) => println!("Saved dna"),
                                Err(_) => println!("Failed to save dna"),
                            }
                        }
                    })

                });

            if !is_open {
                self.selected_ant = None
            }
        }
    }
}

fn load_dna_files() -> Vec<DnaFile> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir("saved_dna") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("dna") {
                if let Ok(bytes) = std::fs::read(&path) {
                    match Brain::from_bytes(&bytes) {
                        Ok(brain) => {
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            files.push(DnaFile { name, brain });
                            println!("Loaded dna: {:#?}", &path);
                        }
                        Err(e) => println!("Failed to parse DNA file {:#?}: {}", path, e)
                    }
                }
            }
        }
    }

    files
}