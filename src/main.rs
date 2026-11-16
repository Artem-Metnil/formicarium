mod simulation;
mod field;
mod ant;
mod brain;
mod graph_brain;

use macroquad::prelude::*;
use egui_macroquad::{egui, ui};
use crate::ant::{ReproductionType, MAX_ENERGY};
use crate::simulation::SimulationWorker;

pub const DEFAULT_INTERVAL: u32 = 10;
const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const GUI_WINDOW_WIDTH: f32 = 280.0;
const FIELD_OBJECT_SIZE: f32 = 8.0;

const FIELD_WIDTH: usize = ((WINDOW_WIDTH as f32 - GUI_WINDOW_WIDTH) / FIELD_OBJECT_SIZE) as usize;
const FIELD_HEIGHT: usize = (WINDOW_HEIGHT as f32 / FIELD_OBJECT_SIZE) as usize;

#[derive(PartialEq, Eq)]
enum RenderType {
    Natural,
    Predators,
    Energy,
    Reproduction,
}


fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Formicarium"),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut simulation = SimulationWorker::new(FIELD_WIDTH, FIELD_HEIGHT);

    let mut theme_initialized = false;
    let mut interval_ms = DEFAULT_INTERVAL;
    let mut is_stopped = true;
    let mut render_mode = RenderType::Natural;


    loop {
        let snapshot = simulation.get_snapshot();

        clear_background(WHITE);

        let mut ant_count = 0;
        for ant in snapshot.ants {
            ant_count += 1;
            let pos_x = ant.x as f32 * FIELD_OBJECT_SIZE;
            let pos_y = ant.y as f32 * FIELD_OBJECT_SIZE;

            let color: Color = match render_mode {
                RenderType::Reproduction => {
                    match ant.reproduction_type {
                        ReproductionType::Budding => { GREEN }
                        ReproductionType::Mitosis => { BLUE }
                    }
                },
                RenderType::Predators => {
                    let is_photo = ant.score_photosynthesis > ant.score_herbivorous && ant.score_photosynthesis > ant.score_predators;
                    let is_pred = ant.score_predators > ant.score_herbivorous && ant.score_predators > ant.score_photosynthesis;
                    let is_herb = ant.score_herbivorous > ant.score_predators && ant.score_herbivorous > ant.score_photosynthesis;
                    if is_photo {
                        SKYBLUE
                    } else if is_pred {
                        RED
                    } else if is_herb {
                        GREEN
                    } else {
                        GRAY
                    }
                }

                RenderType::Energy => {
                    let e = ant.energy / MAX_ENERGY;
                    Color::new(1.0, e, 0.0, 1.0)
                }
                _ => {
                    let tag = ant.get_tag();
                    let r = (tag & 0xFF) as u8;
                    let g = (tag >> 8) as u8;
                    let b = (tag >> 16) as u8;
                    Color::from_rgba(r, g, b, 255)
                }
            };
            draw_rectangle(pos_x,pos_y,FIELD_OBJECT_SIZE,FIELD_OBJECT_SIZE, GRAY);
            draw_rectangle(pos_x + 1.0,
                           pos_y + 1.0,
                           FIELD_OBJECT_SIZE - 2.0,
                           FIELD_OBJECT_SIZE - 2.0,
                           color);
        }

        egui_macroquad::ui(|ctx| {

            if !theme_initialized {
                ctx.set_pixels_per_point(2.0);

                let mut visuals = egui::Visuals::dark();
                visuals.override_text_color = Some(egui::Color32::WHITE);
                ctx.set_visuals(visuals);

                theme_initialized = true;
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
                        ui.label(format!("(interval {}, speed {} ticks/sec)", interval_ms, snapshot.actual_ticks_per_second));
                        ui.label(format!("Object total: {}", ant_count));
                        ui.label(format!("Season: {}", snapshot.season));
                    });

                    ui.group(|ui| {
                        let size = ui.available_width();
                        ui.set_width(size);
                        ui.label("Controls");
                        if ui.add_sized([size / 2.0, 0.0], egui::Button::new(if is_stopped {"Start"} else {"Stop"})).clicked() {
                            is_stopped = !is_stopped;
                            simulation.set_paused(is_stopped);
                        }
                        let interval_changed = ui.add_sized([size, 0.0], egui::Slider::new(&mut interval_ms, 100..=0).show_value(false)
                            .show_value(true)
                            .text("interval"))
                            .changed();
                        if interval_changed {
                            simulation.set_interval(interval_ms);
                        }
                    });

                    ui.group(|ui| {
                       ui.set_width(ui.available_width());
                        ui.label("Render");
                        ui.radio_value(&mut render_mode, RenderType::Natural, "Natural");
                        ui.radio_value(&mut render_mode, RenderType::Predators, "Predators");
                        ui.radio_value(&mut render_mode, RenderType::Energy, "Energy");
                        ui.radio_value(&mut render_mode, RenderType::Reproduction, "Reproduction");
                    });
                });
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}
