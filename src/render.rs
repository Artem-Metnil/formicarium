use macroquad::color::{Color, BLUE, GREEN, WHITE, SKYBLUE, RED, GRAY};
use macroquad::prelude::draw_rectangle;
use macroquad::shapes::draw_line;
use macroquad::window::clear_background;
use crate::{FIELD_OBJECT_SIZE};
use crate::ant::{MAX_ENERGY};
use crate::field::DIRECTIONS;
use crate::graph_brain::ReproductionType;
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
                match ant.get_reproduction_type() {
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
                let e = 1.0 - ant.energy / MAX_ENERGY;
                Color::new(1.0, e, 0.0, 1.0)
            }
            _ => {
                let tag = ant.get_tag();
                color_from_tag(tag)
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

fn color_from_tag(tag: u64) -> Color {
    const MASK_1: u64 = 0x1FFFFF;
    const MASK_2: u64 = 0x1FFFFF << 21;
    const MASK_3: u64 = 0x3FFFFF << 42;

    let c1 = (tag & MASK_1).count_ones() as usize;
    let c2 = ((tag & (MASK_2)) >> 21).count_ones() as usize;
    let c3 = ((tag & (MASK_3)) >> 42).count_ones() as usize;

    // Gaussian distribution
    const CDF_21: [f32; 22] = [
        0.1500, 0.1500, 0.1501, 0.1505, 0.1525, 0.1593, 0.1774, 0.2162,
        0.2842, 0.3823, 0.5000, 0.6177, 0.7158, 0.7838, 0.8226, 0.8407,
        0.8475, 0.8495, 0.8499, 0.8500, 0.8500, 0.8500,
    ];

    const CDF_22: [f32; 23] = [
        0.1500, 0.1500, 0.1500, 0.1502, 0.1512, 0.1548, 0.1653, 0.1883,
        0.2286, 0.2869, 0.3563, 0.4307, 0.5000, 0.5693, 0.6437, 0.7131,
        0.7714, 0.8117, 0.8347, 0.8452, 0.8488, 0.8498, 0.8500,
    ];

    let r = CDF_21[c1];
    let g = CDF_21[c2];
    let b = CDF_22[c3];
    Color::new(r, g, b, 1.0)
}