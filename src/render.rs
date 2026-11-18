use macroquad::color::{Color, BLUE, GREEN, WHITE, SKYBLUE, RED, GRAY};
use macroquad::prelude::draw_rectangle;
use macroquad::shapes::draw_line;
use macroquad::window::clear_background;
use crate::{FIELD_OBJECT_SIZE};
use crate::ant::{ReproductionType, MAX_ENERGY};
use crate::field::DIRECTIONS;
use crate::simulation::SimulationSnapshot;

#[derive(PartialEq, Eq)]
pub enum RenderType {
    Natural,
    Predators,
    Energy,
    Reproduction,
}

pub struct RenderOptions {
    pub mode: RenderType,
}

impl Default for RenderOptions {
    fn default() -> RenderOptions {
        Self {
            mode: RenderType::Natural,
        }
    }
}

pub fn render_field(snapshot: &SimulationSnapshot, options: &RenderOptions) {
    clear_background(WHITE);

    for ant in &snapshot.ants {
        let pos_x = ant.x as f32 * FIELD_OBJECT_SIZE;
        let pos_y = ant.y as f32 * FIELD_OBJECT_SIZE;

        let color: Color = match options.mode {
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
        draw_rectangle(pos_x, pos_y, FIELD_OBJECT_SIZE, FIELD_OBJECT_SIZE, GRAY);
        draw_rectangle(pos_x + 1.0,
                       pos_y + 1.0,
                       FIELD_OBJECT_SIZE - 2.0,
                       FIELD_OBJECT_SIZE - 2.0,
                       color);
        if options.mode == RenderType::Natural {
            let center_x = pos_x + FIELD_OBJECT_SIZE / 2.0;
            let center_y = pos_y + FIELD_OBJECT_SIZE / 2.0;
            let (dx, dy) = DIRECTIONS[ant.direction as usize];
            let end_x = center_x + dx as f32 * (FIELD_OBJECT_SIZE / 2.0);
            let end_y = center_y + dy as f32 * (FIELD_OBJECT_SIZE / 2.0);
            draw_line(center_x,center_y, end_x, end_y, 1.0, GRAY);
        }
    }
}