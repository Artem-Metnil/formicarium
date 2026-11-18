use rand::Rng;
use crate::graph_brain::{Brain, NUM_INPUTS, NUM_OUTPUTS, ReproductionType};

pub const MAX_LIFE_TIME: u32 = 500;
pub const MAX_ENERGY: f32 = 250.0;

// BRAIN INPUTS
pub const IN_ENERGY:        usize = 0;
pub const IN_DIRECTION:     usize = 1;
pub const IN_HEIGHT:        usize = 2;
pub const IN_EYE:           usize = 3;
pub const IN_EYE_RELATIVE:  usize = 4;
pub const IN_X:           usize = 5;
pub const IN_DETRITUS:      usize = 6;
pub const IN_AGE:           usize = 7;
// BRAIN OUTPUTS
pub const OUT_PHOTOSYNTHESIS: usize = 0;
pub const OUT_EAT:            usize = 1;
pub const OUT_MOVE:           usize = 2;
pub const OUT_ROTATE_LEFT:    usize = 3;
pub const OUT_ROTATE_RIGHT:   usize = 4;
pub const OUT_ATTACK:         usize = 5;
pub const OUT_REPRODUCE:      usize = 6;

#[derive(Clone, Copy)]
pub enum Action {
    Attack,
    Eat,
    Move,
    Photosynthesis,
    Reproduce,
    RotateLeft,
    RotateRight,
    Idle,
}


impl Action {
    pub fn cost(&self) -> f32 {
        match self {
            Action::Attack => 10f32,
            Action::Eat => 10f32,
            Action::Move => 1f32,
            Action::Photosynthesis => 0f32,
            Action::Reproduce => 50f32,
            Action::RotateLeft => 0f32,
            Action::RotateRight => 0f32,
            Action::Idle => 0f32,
        }
    }

    pub fn from_idx(idx: usize) -> Action {
        match idx {
            OUT_ATTACK => Action::Attack,
            OUT_EAT => Action::Eat,
            OUT_MOVE => Action::Move,
            OUT_PHOTOSYNTHESIS => Action::Photosynthesis,
            OUT_REPRODUCE => Action::Reproduce,
            OUT_ROTATE_LEFT => Action::RotateLeft,
            OUT_ROTATE_RIGHT => Action::RotateRight,
            _ => Action::Idle,
        }
    }
}

#[derive(Clone)]
pub struct Ant {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub direction: i8,
    pub age: u32,
    pub want_to_do: Action,

    pub score_photosynthesis: f32,
    pub score_predators: f32,
    pub score_herbivorous: f32,

    brain: Brain
}

impl Ant {
    pub fn new(x: usize, y: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x,
            y,
            energy: rng.gen_range(50.0..MAX_ENERGY),
            direction: 0,
            age: 0,
            want_to_do: Action::Idle,

            score_photosynthesis: 0.0,
            score_predators: 0.0,
            score_herbivorous: 0.0,
            brain: Brain::new(&mut rng)
        }
    }

    pub fn think(&mut self, input: [f32; NUM_INPUTS as usize]) {
        self.energy -= self.brain.metabolic_cost();
        let brain_outputs = self.brain.forward(input);

        let mut best_idx = usize::MAX;
        let mut best_val = 0.5f32;
        for i in 0..NUM_OUTPUTS as usize {
            if brain_outputs[i] > best_val {
                best_idx = i;
                best_val = brain_outputs[i];
            }
        }

        if best_idx == usize::MAX {
            self.want_to_do = Action::Idle;
        } else {
            self.want_to_do = Action::from_idx(best_idx);
        }
    }

    pub fn photosynthesis(&mut self, energy: f32) {
        self.energy += energy;
        self.score_photosynthesis += energy;
        if self.energy > MAX_ENERGY {
            self.energy = MAX_ENERGY;
        }
    }

    pub fn hunt_energy(&mut self, energy: f32) {
        self.energy += energy;
        self.score_predators += energy;
        if self.energy > MAX_ENERGY {
            self.energy = MAX_ENERGY;
        }
    }

    pub fn eat_energy(&mut self, energy: f32) {
        self.energy += energy;
        self.score_herbivorous += energy;
        if self.energy > MAX_ENERGY {
            self.energy = MAX_ENERGY;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.energy > 0.0  && self.age < MAX_LIFE_TIME
    }

    pub fn divide_budding(&self, x: usize, y: usize, energy: f32) -> Self {
        let mut rng = rand::thread_rng();
        let child_brain = self.brain.clone();

        let mut child = Self {
            x,
            y,
            energy,
            direction: 0,
            age: 0,
            want_to_do: Action::Idle,

            score_photosynthesis: 0.0,
            score_predators: 0.0,
            score_herbivorous: 0.0,
            brain: child_brain
        };

        if rng.gen_bool(0.20) {
            child.mutate(&mut rng);
        }

        child
    }

    pub fn divide_mitosis(&self, x: usize, y: usize, energy: f32) -> Self {
        let mut rng = rand::thread_rng();
        let child_brain = self.brain.clone();
        let mut child = Self {
            x,
            y,
            energy,
            direction: 0,
            age: 0,
            want_to_do: Action::Idle,

            score_photosynthesis: 0.0,
            score_predators: 0.0,
            score_herbivorous: 0.0,
            brain: child_brain
        };

        if rng.gen_bool(0.05) {
            child.mutate(&mut rng);
        }

        child
    }

    pub fn get_tag(&self) -> usize {
        self.brain.tag
    }
    pub fn is_relative(&self, other_tag: usize) -> bool {
        let diff = self.brain.tag ^ other_tag;
        diff.count_ones() < 5
    }
    pub fn get_reproduction_type(&self) -> ReproductionType {
        self.brain.reproduction_type
    }
    pub fn get_brain_copy(&self) -> Brain {
        self.brain.clone()
    }
    fn mutate(&mut self, rng: &mut impl Rng) {
        self.brain.mutate(rng);
    }
}