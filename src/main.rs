mod simulation;
mod field;
mod ant;
mod graph_brain;
mod render;
mod gui;

use macroquad::prelude::*;
use crate::gui::Gui;
use crate::render::{render_field};
use crate::simulation::{SimulationWorker};

pub const DEFAULT_INTERVAL: u32 = 10;
const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const GUI_WINDOW_WIDTH: f32 = 280.0;
const FIELD_OBJECT_SIZE: f32 = 8.0;

const FIELD_WIDTH: usize = ((WINDOW_WIDTH as f32 - GUI_WINDOW_WIDTH) / FIELD_OBJECT_SIZE) as usize;
const FIELD_HEIGHT: usize = (WINDOW_HEIGHT as f32 / FIELD_OBJECT_SIZE) as usize;


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
    let mut gui = Gui::default();

    loop {
        let snapshot = simulation.get_snapshot();
        render_field(&snapshot, &gui.render_options);
        egui_macroquad::ui(|ctx| {
            gui.draw_gui(&ctx, &snapshot, &mut simulation);
        });
        egui_macroquad::draw();
        next_frame().await;
    }
}
