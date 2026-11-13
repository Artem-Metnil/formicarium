use std::vec::Vec;
use rand::Rng;
use crate::ant::Ant;

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
            let ant = self.ants[i];
            self.move_ant(i, ant.x + 1, ant.y);
        }
    }

    pub fn get_ants(&self) -> &Vec<Ant> {
        &self.ants
    }

    fn move_ant(&mut self, ant_index : usize, x : usize, y : usize) {
        if x >= self.width || y >= self.height {
            return;
        }

        let target_idx = x + y * self.width;
        if let Object::Null = self.all_cells[target_idx] {
            let old_x = self.ants[ant_index].x;
            let old_y = self.ants[ant_index].y;
            let current_idx = old_x + old_y * self.width;

            self.all_cells[current_idx] = Object::Null;
            self.all_cells[target_idx] = Object::Ant { id: ant_index };

            self.ants[ant_index].x = x;
            self.ants[ant_index].y = y;
        }
    }
}