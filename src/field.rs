use std::vec::Vec;
use rand::Rng;
use crate::ant::{Ant, ReproductionType, ENERGY_FOR_REPRODUCE, IN_DIRECTION, IN_ENERGY, IN_EYE, IN_EYE_RELATIVE, IN_HEIGHT, IN_MEMORY, IN_SEX, MAX_ENERGY, ENERGY_FOR_HUNT, IN_DETRITUS, ENERGY_FOR_EAT, IN_AGE, MAX_LIFE_TIME};
use crate::FIELD_HEIGHT;

const SEASON_DURATION: usize = 1000;

pub const DIRECTIONS: [(isize, isize); 8] = [
    (0, -1), // 0: Вверх
    (1, -1), // 1: Вправо-вверх
    (1, 0),  // 2: Вправо
    (1, 1),  // 3: Вправо-вниз
    (0, 1),  // 4: Вниз
    (-1, 1), // 5: Влево-вниз
    (-1, 0), // 6: Влево
    (-1, -1) // 7: Влево-вверх
];

#[derive(Clone)]
enum Object {
    Null,
    Ant { id: usize }
}

#[derive(Eq, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter
}

pub struct Field {
    width: usize,
    height: usize,
    season_timer: usize,
    season: Season,
    all_cells: Vec<Object>,
    ants: Vec<Ant>,
    detritus_layer: Vec<f32>
}

impl Field {
    pub fn new(width : usize, height : usize) -> Self {
        let mut all_cells = vec![Object::Null; width * height];
        let detritus_layer = vec![0.0; width * height];

        let target_ants = 1000.min(width * height);
        let mut ants = Vec::with_capacity(target_ants);
        let mut rng = rand::thread_rng();

        let mut spawned = 0;
        while spawned < target_ants {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            let idx = x + y * width;

            if let Object::Null = all_cells[idx] {
                all_cells[idx] = Object::Ant { id: spawned };
                ants.push(Ant::new(x,y));
                spawned += 1;
            }
        }


        Self {
            width,
            height,
            season_timer: 0,
            season: Season::Spring,
            all_cells,
            ants,
            detritus_layer
        }
    }

    pub fn tick(&mut self) {
        let light_multiply = self.season_tick();

        for i in 0..self.ants.len() {
            let input = self.compute_input(i);
            self.ants[i].think(&input)
        }

        let ant_count = self.ants.len();
        for i in 0..ant_count {
            if !self.ants[i].is_alive() {
                continue;
            }

            if self.ants[i].want_reproduce {
                self.try_reproduce(i);
            }

            if !self.ants[i].is_alive() {
                continue;
            }

            if self.ants[i].want_attack {
                self.try_hunt(i);
            } else if self.ants[i].want_eat {
                self.try_eat(i);
            } else if self.ants[i].want_move {
                self.move_ant(i);
                self.ants[i].energy -= 1.0;
            } else if self.ants[i].want_photosynthesis {
                self.ants[i].photosynthesis(light_multiply);
            }
        }

        let mut i = 0;
        while i < self.ants.len() {
            if !self.ants[i].is_alive() {
                self.kill_ant(i);
            } else {
                i += 1
            }
        }
    }

    pub fn get_ants(&self) -> &Vec<Ant> {
        &self.ants
    }
    pub fn get_season(&self) -> String {
        match self.season {
            Season::Spring => String::from("Spring"),
            Season::Summer => String::from("Summer"),
            Season::Autumn => String::from("Autumn"),
            Season::Winter => String::from("Winter")
        }
    }
    fn move_ant(&mut self, ant_index : usize) {
        let (dx, dy) = DIRECTIONS[self.ants[ant_index].direction as usize];

        let old_x = self.ants[ant_index].x;
        let old_y = self.ants[ant_index].y;
        let x = old_x as isize + dx;
        let y = old_y as isize + dy;

        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return;
        }

        let x = x as usize;
        let y = y as usize;

        let target_idx = x + y * self.width;
        if let Object::Null = self.all_cells[target_idx] {

            let current_idx = old_x + old_y * self.width;

            self.all_cells[current_idx] = Object::Null;
            self.all_cells[target_idx] = Object::Ant { id: ant_index };

            self.ants[ant_index].x = x;
            self.ants[ant_index].y = y;
            self.ants[ant_index].memory = 1.0
        }
    }

    fn season_tick(&mut self) -> f32 {
        self.season_timer += 1;
        if self.season_timer >= SEASON_DURATION {
            self.season_timer = 0;
            self.season = match self.season {
                Season::Spring => Season::Summer,
                Season::Summer => Season::Autumn,
                Season::Autumn => Season::Winter,
                Season::Winter => Season::Spring
            }
        }
        match self.season {
            Season::Spring => { 0.75 }
            Season::Summer => { 1.0 }
            Season::Autumn => { 0.45 }
            Season::Winter => { 0.05 }
        }
    }

    fn compute_input(&self, ant_index : usize) -> [f32; 9] {
        let mut brain_input = [0.0f32; 9];
        brain_input[IN_ENERGY] = self.ants[ant_index].energy / MAX_ENERGY;
        brain_input[IN_DIRECTION] = self.ants[ant_index].direction as f32 / 8.0;
        brain_input[IN_HEIGHT] = self.ants[ant_index].y as f32 / FIELD_HEIGHT as f32;
        let (eye, relative) = self.see_forward(ant_index);
        brain_input[IN_EYE] = eye;
        brain_input[IN_EYE_RELATIVE] = relative;
        brain_input[IN_SEX] = 0.0;
        let idx = self.ants[ant_index].x + self.ants[ant_index].y * self.width;
        brain_input[IN_DETRITUS] = (self.detritus_layer[idx] / 50.0).min(1.0);
        brain_input[IN_MEMORY] = self.ants[ant_index].memory;
        brain_input[IN_AGE] = self.ants[ant_index].age as f32 / MAX_LIFE_TIME as f32;
        brain_input
    }

    fn see_forward(&self, ant_index : usize) -> (f32, f32) {
        let ant = &self.ants[ant_index];
        let (dx, dy) = DIRECTIONS[ant.direction as usize];

        let x = ant.x as isize + dx;
        let y = ant.y as isize + dy;
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return (-1.0, 0.0);
        }

        let idx = x as usize + y as usize * self.width;
        match self.all_cells[idx] {
            Object::Null => (0.0, 0.0),
            Object::Ant { id } => {
                let is_relative = self.ants[ant_index].is_relative(self.ants[id].get_tag());
                if is_relative {
                    (1.0, 1.0)
                } else {
                    (1.0, -1.0)
                }
            },
        }
    }

    fn kill_ant(&mut self, ant_index : usize) {
        let dead = &self.ants[ant_index];
        let dead_idx = dead.x + dead.y * self.width;
        self.all_cells[dead_idx] = Object::Null;

        if dead.energy > 0f32 {
            self.detritus_layer[dead_idx] += 0.8 * dead.energy;
        }

        let last_idx = self.ants.len() - 1;
        if ant_index != last_idx {
            let last_ants = &self.ants[last_idx];
            let last_ants_idx = last_ants.x + last_ants.y * self.width;
            self.all_cells[last_ants_idx] = Object::Ant { id: ant_index };
        }

        self.ants.swap_remove(ant_index);
    }

    fn try_reproduce(&mut self, ant_index : usize) {
        if self.ants[ant_index].energy < ENERGY_FOR_REPRODUCE {
            return;
        }

        let pos_x = self.ants[ant_index].x;
        let pos_y = self.ants[ant_index].y;

        if self.ants[ant_index].reproduction_type == ReproductionType::Budding {
            for &(dx, dy) in DIRECTIONS.iter() {
                let target_x = pos_x as isize + dx;
                let target_y = pos_y as isize + dy;
                if target_x < 0 || target_x >= self.width as isize || target_y < 0 || target_y >= self.height as isize {
                    continue;
                }

                let target_x = target_x as usize;
                let target_y = target_y as usize;
                let idx = target_x + target_y * self.width;

                if let Object::Null = self.all_cells[idx] {
                    self.ants[ant_index].energy -= ENERGY_FOR_REPRODUCE;
                    let energy = self.ants[ant_index].energy * 0.25;
                    self.ants[ant_index].energy -= energy;
                    let child = self.ants[ant_index].divide_budding(target_x, target_y, energy);
                    let new_id = self.ants.len();
                    self.ants.push(child);
                    self.all_cells[idx] = Object::Ant { id: new_id };
                    return;
                }
            }
        } else if self.ants[ant_index].reproduction_type == ReproductionType::Mitosis {
            let self_x = self.ants[ant_index].x;
            let self_y = self.ants[ant_index].y;

            let mut first_cell: Option<(usize, usize, usize)> = None;
            let mut second_cell: Option<(usize, usize, usize)> = None;

            for &(dx, dy) in DIRECTIONS.iter() {
                let target_x = self_x as isize + dx;
                let target_y = self_y as isize + dy;

                if target_x < 0 || target_x >= self.width as isize || target_y < 0 || target_y >= self.height as isize {
                    continue;
                }
                let target_x = target_x as usize;
                let target_y = target_y as usize;
                let idx = target_x + target_y * self.width;
                if let Object::Null = self.all_cells[idx] {
                    if first_cell.is_none() {
                        first_cell = Some((idx, target_x, target_y));
                    } else if second_cell.is_none() {
                        second_cell = Some((idx, target_x, target_y));
                    }
                }
            }

            if let (Some(c1), Some(c2)) = (first_cell, second_cell) {
                let energy = self.ants[ant_index].energy * 0.5;
                self.ants[ant_index].energy = 0f32;

                let child1 = self.ants[ant_index].divide_mitosis(c1.1, c1.2, energy);
                let child2 = self.ants[ant_index].divide_mitosis(c2.1, c2.2, energy);

                let new_id = self.ants.len();
                self.ants.push(child1);
                self.all_cells[c1.0] = Object::Ant { id: new_id };
                let new_id = self.ants.len();
                self.ants.push(child2);
                self.all_cells[c2.0] = Object::Ant { id: new_id };
            }
        }
    }

    fn try_hunt(&mut self, ant_index : usize) {
        if self.ants[ant_index].energy < ENERGY_FOR_HUNT {
            return;
        }

        let (dx, dy) = DIRECTIONS[self.ants[ant_index].direction as usize];
        let x = self.ants[ant_index].x as isize + dx;
        let y = self.ants[ant_index].y as isize + dy;
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        let idx = x + y * self.width;

        if let Object::Ant { id: enemy_index } = self.all_cells[idx] {
            if self.ants[enemy_index].is_alive() {
                let energy = self.ants[enemy_index].energy;
                let needed = MAX_ENERGY - self.ants[ant_index].energy;
                let eaten = energy.min(needed);

                self.ants[enemy_index].energy = 0f32;
                self.ants[ant_index].hunt_energy(eaten);
                self.ants[ant_index].energy -= ENERGY_FOR_HUNT;
                self.ants[ant_index].memory = 1.0;

                let left_over = energy - eaten;
                if left_over > 0f32 {
                    let idx = self.ants[enemy_index].x + self.ants[enemy_index].y * self.width;
                    self.detritus_layer[idx] += left_over;
                }
            }
        }
    }

    fn try_eat(&mut self, ant_index : usize) {
        if self.ants[ant_index].energy < ENERGY_FOR_EAT {
            return;
        }
        let idx = self.ants[ant_index].x + self.ants[ant_index].y * self.width;
        let available = self.detritus_layer[idx];

        if available <= 0.0 {
            self.ants[ant_index].memory = -1.0;
            return;
        }

        let needed = MAX_ENERGY - self.ants[ant_index].energy;
        let eaten = available.min(needed);
        if eaten > 0.0 {
            self.detritus_layer[idx] -= eaten;
            self.ants[ant_index].energy -= ENERGY_FOR_EAT;
            self.ants[ant_index].eat_energy(eaten);
            self.ants[ant_index].memory = 1.0;
        }

    }
}