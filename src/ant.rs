#[derive(Copy, Clone)]
pub struct Ant {
    pub x: usize,
    pub y: usize,
}

impl Ant {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, }
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }
}