use macroquad::prelude::*;
use egui_macroquad::{egui, ui};

const GUI_WINDOW_WIDTH: f32 = 284.0;


fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Formicarium"),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {

    let mut theme_initialized = false;
    let mut slider_value = 0;
    let mut is_stopped = false;

    loop {
        clear_background(GRAY);
        egui_macroquad::ui(|ctx| {

            if !theme_initialized {
                ctx.set_pixels_per_point(2.0);

                let mut visuals = egui::Visuals::dark();
                visuals.override_text_color = Some(egui::Color32::WHITE);
                ctx.set_visuals(visuals);

                theme_initialized = true;
            }

            draw_rectangle(100.0, 100.0, 50.0, 50.0, RED);

            egui::SidePanel::right("control_panel")
                .exact_width(GUI_WINDOW_WIDTH)
                .resizable(false)
                .frame(egui::Frame::side_top_panel(&ctx.style()).fill(egui::Color32::BLACK))
                .show(ctx, |ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                       ui.label("Main");
                        ui.label("steps: 0");
                        ui.label("(interval 0, speed 0 ticks/sec)");
                        ui.label("Object total: 0");
                    });

                    ui.group(|ui| {
                        let size = ui.available_width();
                        ui.set_width(size);
                        ui.label("Controls");
                        if ui.add_sized([size / 2.0, 0.0], egui::Button::new(if is_stopped {"Start"} else {"Stop"})).clicked() {
                            is_stopped = !is_stopped;
                        }
                        ui.add_sized([size, 0.0], egui::Slider::new(&mut slider_value, 120..=0).show_value(false).show_value(true).text("interval"));
                    });
                });
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}
