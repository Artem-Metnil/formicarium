use std::vec::Vec;
use rand::Rng;
use crate::ant::{Ant, IN_DIRECTION, IN_ENERGY, IN_EYE, IN_EYE_RELATIVE, IN_HEIGHT, IN_MEMORY, IN_SEX, MAX_ENERGY};
use crate::FIELD_HEIGHT;

#[derive(Clone)]
enum Object {
    Null,
    Ant { id: usize }
}

pub struct Field {
    width: usize,
    height: usize,
    all_cells: Vec<Object>,
    ants: Vec<Ant>,
}

impl Field {
    pub fn new(width : usize, height : usize) -> Self {
        let mut all_cells = vec![Object::Null; width * height];

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
            all_cells,
            ants
        }
    }

    pub fn tick(&mut self) {
        for i in 0..self.ants.len() {
            let input = self.compute_input(i);
            self.ants[i].think(&input)
        }

        let ant_count = self.ants.len();
        for i in 0..ant_count {
            if self.ants[i].energy <= 0f32 {
                continue;
            }

            if self.ants[i].want_reproduce {
                // REPRODUCE
            }

            if self.ants[i].want_attack {
                // ATTACK
            } else if self.ants[i].want_move {
                self.move_ant(i);
                self.ants[i].energy -= 1.0;
            } else if self.ants[i].want_photosynthesis {
                self.ants[i].photosynthesis()
            }
        }

        let mut i = 0;
        while i < self.ants.len() {
            if self.ants[i].energy <= 0f32 {
                self.kill_ant(i);
            }
            i += 1
        }
    }

    pub fn get_ants(&self) -> &Vec<Ant> {
        &self.ants
    }

    fn move_ant(&mut self, ant_index : usize) {
        let (dx, dy) = self.ants[ant_index].get_direction();

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

    fn compute_input(&self, ant_index : usize) -> [f32; 7] {
        let mut brain_input = [0.0f32; 7];
        brain_input[IN_ENERGY] = self.ants[ant_index].energy / MAX_ENERGY;
        brain_input[IN_DIRECTION] = self.ants[ant_index].direction as f32 / 8.0;
        brain_input[IN_HEIGHT] = self.ants[ant_index].y as f32 / FIELD_HEIGHT as f32;
        brain_input[IN_EYE] = self.see_forward(ant_index);
        brain_input[IN_EYE_RELATIVE] = 0.0;
        brain_input[IN_SEX] = 0.0;
        brain_input[IN_MEMORY] = self.ants[ant_index].memory;
        brain_input
    }

    fn see_forward(&self, ant_index : usize) -> f32 {
        let ant = &self.ants[ant_index];
        let (dx, dy) = ant.get_direction();

        let x = ant.x as isize + dx;
        let y = ant.y as isize + dy;
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return -1.0;
        }

        let idx = x as usize + y as usize * self.width;
        match self.all_cells[idx] {
            Object::Null => 0.0,
            Object::Ant {..} => 1.0,
        }
    }

    fn kill_ant(&mut self, ant_index : usize) {
        let dead = &self.ants[ant_index];
        let dead_idx = dead.x + dead.y * self.width;
        self.all_cells[dead_idx] = Object::Null;

        let last_idx = self.ants.len() - 1;
        if ant_index != last_idx {
            let last_ants = &self.ants[last_idx];
            let last_ants_idx = last_ants.x + last_ants.y * self.width;
            self.all_cells[last_ants_idx] = Object::Ant { id: ant_index };
        }

        self.ants.swap_remove(ant_index);
    }
}