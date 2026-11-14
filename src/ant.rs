use rand::Rng;
use crate::brain::Brain;
use crate::FIELD_HEIGHT;

const MAX_LIFE_TIME: u32 = 200;
pub const MAX_ENERGY: f32 = 500.0;
pub const MIN_ENERGY_FOR_REPRODUCE: f32 = 50.0;
const PHOTOSYNTHESIS_ENERGY: f32 = 5.0;

pub const IN_ENERGY: usize = 0;
pub const IN_DIRECTION: usize = 1;
pub const IN_HEIGHT: usize = 2;
pub const IN_EYE: usize = 3;
pub const IN_EYE_RELATIVE: usize = 4;
pub const IN_SEX: usize = 5;
pub const IN_MEMORY: usize = 6;

const OUT_ROTATE_LEFT: usize = 0;
const OUT_ROTATE_RIGHT: usize = 1;
const OUT_MOVE: usize = 2;
const OUT_PHOTOSYNTHESIS: usize = 3;
const OUT_ATTACK: usize = 4;
const OUT_REPRODUCE: usize = 5;


#[derive(Clone)]
pub struct Ant {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub direction: i8,
    pub memory: f32,
    pub age: u32,

    pub want_attack: bool,
    pub want_move: bool,
    pub want_photosynthesis: bool,
    pub want_reproduce: bool,

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
            memory: 0.0,
            age: 0,

            want_attack: false,
            want_move: false,
            want_reproduce: false,
            want_photosynthesis: false,
            brain: Brain::new()
        }
    }

    pub fn think(&mut self, input: &[f32; 7]) {

        let brain_outputs = self.brain.forward(&input);

        if brain_outputs[OUT_ROTATE_LEFT] > 0.5f32 {
            self.direction -= 1;
            if self.direction < 0 {
                self.direction = 7;
            }
        }
        if brain_outputs[OUT_ROTATE_RIGHT] > 0.5f32 {
            self.direction += 1;
            if self.direction > 7 {
                self.direction = 0;
            }
        }

        self.want_attack = brain_outputs[OUT_ATTACK] > 0.5f32;
        self.want_move = brain_outputs[OUT_MOVE] > 0.5f32;
        self.want_photosynthesis = brain_outputs[OUT_PHOTOSYNTHESIS] > 0.5f32;
        self.want_reproduce = brain_outputs[OUT_REPRODUCE] > 0.5f32;
        self.memory = 0.0;
        self.age += 1;
    }

    pub fn photosynthesis(&mut self) {
        self.energy += PHOTOSYNTHESIS_ENERGY;
        if self.energy > MAX_ENERGY {
            self.energy = MAX_ENERGY;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.energy > 0.0 && self.age <= MAX_LIFE_TIME
    }

    pub fn divide(&self, x: usize, y: usize, energy: f32) -> Self {
        let mut rng = rand::thread_rng();
        let mut child_brain = self.brain.clone();

        if rng.gen_bool(0.20) {
            let n = rng.gen_range(1..7);
            for _ in 0..n {
                child_brain.mutate();
            }
        }

        Self {
            x,
            y,
            energy,
            direction: 0,
            memory: 0.0,
            age: 0,
            want_attack: false,
            want_move: false,
            want_reproduce: false,
            want_photosynthesis: false,
            brain: child_brain
        }
    }
    pub fn get_tag(&self) -> usize { self.brain.tag }
}