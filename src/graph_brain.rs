use macroquad::math::u64;
use rand::Rng;

pub const NUM_INPUTS: u16 = 8;
pub const NUM_OUTPUTS: u16 = 7;
const NUM_BASE_NODES: u16 = NUM_INPUTS + NUM_OUTPUTS;
const MAGIC: [u8;4] = *b"FROM";
const FORMAT_VERSION: u16 = 1;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ReproductionType {
    Budding,
    Mitosis,
}

#[derive(Copy, Clone)]
pub enum Activation {
    Linear,
    Step,
    Sigmoid,
    Tanh,
}

// 8 bytes
#[derive(Copy, Clone)]
pub struct Edge {
    pub from: u16,
    pub to: u16,
    pub weight: f32,
}

#[derive(Clone)]
pub struct SensorMask(u16);

impl SensorMask {
    pub fn all() -> Self { Self(0xFF) }
    pub fn has(&self, idx: u8) -> bool { (self.0 >> idx) & 1 == 1 }
    pub fn cost(&self) -> f32 {
        let mut cost = 0.0f32;
        for i in 0..NUM_INPUTS {
            if self.has(i as u8) { cost += 0.01; }
        }
        cost
    }

    pub fn bits(&self) -> u16 { self.0 }
    pub fn from_bits(bits: u16) -> Self { Self(bits) }
}

#[derive(Clone)]
pub struct Brain {
    pub tag: u64,
    pub reproduction_type: ReproductionType,
    pub sensors: SensorMask,

    pub bias: Vec<f32>,
    pub activation: Vec<Activation>,
    pub edges: Vec<Edge>,

    pub state: Vec<f32>,
    pub prev_state: Vec<f32>,
    pub eval_order: Vec<u16>,
    pub cycle_nodes: Vec<u16>,
    dirty: bool,
}

impl Brain {
    pub fn new(rng: &mut impl Rng) -> Brain {
        let base = NUM_BASE_NODES as usize;
        let bias = vec![0f32; base];
        let mut activation = Vec::with_capacity(base);

        for _ in 0..NUM_INPUTS {
            activation.push(Activation::Linear);
        }

        for _ in 0..NUM_OUTPUTS {
            activation.push(Activation::Step);
        }

        let mut edges = Vec::with_capacity(3);
        let initial = rng.gen_range(0..=5);

        for _ in 0..initial {
            let from = rng.gen_range(0..NUM_INPUTS);
            let to = NUM_INPUTS + rng.gen_range(0..NUM_OUTPUTS);
            edges.push(Edge {
                from,
                to,
                weight: rng.gen_range(-1.0..1.0),
            });
        }

        Self {
            tag: rng.r#gen(),
            reproduction_type: if rng.gen_bool(0.5) { ReproductionType::Mitosis } else { ReproductionType::Budding },
            sensors: SensorMask::all(),
            bias,
            activation,
            edges,
            state: vec![0f32; base],
            prev_state: vec![0f32; base],
            eval_order: vec![0u16; base],
            cycle_nodes: vec![0u16; 0],
            dirty: true
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256);

        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());

        buf.extend_from_slice(&(self.tag as u64).to_le_bytes());
        buf.push(match self.reproduction_type {
            ReproductionType::Budding => 0,
            ReproductionType::Mitosis => 1,
        });
        buf.extend_from_slice(&self.sensors.bits().to_le_bytes());

        buf.extend_from_slice(&(self.bias.len() as u32).to_le_bytes());
        for &b in &self.bias {
            buf.extend_from_slice(&b.to_le_bytes());
        }

        buf.extend_from_slice(&(self.activation.len() as u32).to_le_bytes());
        for &a in &self.activation {
            buf.push(match a {
                Activation::Linear => 0,
                Activation::Step => 1,
                Activation::Sigmoid => 2,
                Activation::Tanh => 3,
            });
        }

        buf.extend_from_slice(&(self.edges.len() as u32).to_le_bytes());
        for e in &self.edges {
            buf.extend_from_slice(&e.from.to_le_bytes());
            buf.extend_from_slice(&e.to.to_le_bytes());
            buf.extend_from_slice(&e.weight.to_le_bytes());
        }

        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 6 || bytes[0..4] != MAGIC {
            return Err("Invalid file format".to_string());
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != FORMAT_VERSION {
            return Err("Unsupported file version".to_string());
        }

        let mut pos = 6usize;

        let read_u8 = |pos: &mut usize| -> Result<u8, String> {
            if *pos + 1 > bytes.len() { return Err("EOF".to_string()); }
            let v = bytes[*pos];
            *pos += 1;
            Ok(v)
        };

        let read_u16 = |pos: &mut usize| -> Result<u16, String> {
            if *pos + 2 > bytes.len() { return Err("EOF".to_string()); }
            let arr: [u8; 2] = bytes[*pos..*pos + 2].try_into().unwrap();
            *pos += 2;
            Ok(u16::from_le_bytes(arr))
        };

        let read_u32 = |pos: &mut usize| -> Result<u32, String> {
            if *pos + 4 > bytes.len() { return Err("EOF".to_string()); }
            let arr: [u8; 4] = bytes[*pos..*pos + 4].try_into().unwrap();
            *pos += 4;
            Ok(u32::from_le_bytes(arr))
        };

        let read_u64 = |pos: &mut usize| -> Result<u64, String> {
            if *pos + 8 > bytes.len() { return Err("EOF".to_string()); }
            let arr: [u8; 8] = bytes[*pos..*pos + 8].try_into().unwrap();
            *pos += 8;
            Ok(u64::from_le_bytes(arr))
        };

        let read_f32 = |pos: &mut usize| -> Result<f32, String> {
            if *pos + 4 > bytes.len() { return Err("EOF".to_string()); }
            let arr: [u8; 4] = bytes[*pos..*pos + 4].try_into().unwrap();
            *pos += 4;
            Ok(f32::from_le_bytes(arr))
        };

        let tag = read_u64(&mut pos)?;

        let reproduction_type = match read_u8(&mut pos)? {
            0 => ReproductionType::Budding,
            1 => ReproductionType::Mitosis,
            _=> return Err("Unknown reproduction type".to_string()),
        };

        let sensors = SensorMask::from_bits(read_u16(&mut pos)?);

        let bias_count = read_u32(&mut pos)? as usize;
        let mut bias = Vec::with_capacity(bias_count);
        for _ in 0..bias_count {
            bias.push(read_f32(&mut pos)?);
        }

        let activation_count = read_u32(&mut pos)? as usize;
        let mut activation = Vec::with_capacity(activation_count);
        for _ in 0..activation_count {
            activation.push(match read_u8(&mut pos)? {
                0 => Activation::Linear,
                1 => Activation::Step,
                2 => Activation::Sigmoid,
                3 => Activation::Tanh,
                _ => return Err("Unknown activation".to_string())
            });
        }

        let edge_count = read_u32(&mut pos)? as usize;
        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            let from = read_u16(&mut pos)?;
            let to = read_u16(&mut pos)?;
            let weight = read_f32(&mut pos)?;
            edges.push(Edge {from, to, weight});
        }

        let n = bias.len();
        let brain = Self {
            tag,
            reproduction_type,
            sensors,
            bias,
            activation,
            edges,
            state: vec![0.0; n],
            prev_state: vec![0.0; n],
            eval_order: Vec::with_capacity(n),
            cycle_nodes: Vec::new(),
            dirty: true
        };

        Ok(brain)
    }

    pub fn metabolic_cost(&self) -> f32 {
        let sensors_cost = self.sensors.cost();
        let node_cost = 0.002 * self.bias.len() as f32;
        let edge_cost = 0.001 * self.edges.len() as f32;
        sensors_cost + node_cost + edge_cost
    }

    pub fn forward(&mut self, input: [f32; NUM_INPUTS as usize]) -> [f32; NUM_OUTPUTS as usize] {
        if self.dirty {
            self.rebuild_topology();
            self.dirty = false;
        }

        std::mem::swap(&mut self.state, &mut self.prev_state);

        for i in 0..NUM_INPUTS as usize {
            self.state[i] = if self.sensors.has(i as u8) { input[i] } else { 0f32 };
        }

        for i in NUM_INPUTS as usize..self.state.len() {
            self.state[i] = 0.0;
        }

        for i in 0..self.eval_order.len() {
            let node = self.eval_order[i];
            if is_input(node) { continue; }
            let mut sum = self.bias[node as usize];
            for e in &self.edges {
                if e.to != node { continue; }
                if e.from == e.to {
                    sum += self.prev_state[e.from as usize] * e.weight;
                } else {
                    sum += self.state[e.from as usize] * e.weight;
                }
            }
            self.state[node as usize] = apply_activation(self.activation[node as usize], sum);
        }

        for i in 0..self.cycle_nodes.len() {
            let node = self.cycle_nodes[i];
            if is_input(node) { continue; }
            let mut sum = self.bias[node as usize];
            for e in &self.edges {
                if e.to != node { continue; }
                sum += self.prev_state[e.from as usize] * e.weight;
            }
            self.state[node as usize] = apply_activation(self.activation[node as usize], sum);
        }

        let mut out = [0.0f32; NUM_OUTPUTS as usize];
        for i in 0..NUM_OUTPUTS as usize {
            out[i] = self.state[i + NUM_INPUTS as usize];
        }
        out
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) {

        if rng.gen_bool(0.01) {
            self.reproduction_type = match self.reproduction_type {
                ReproductionType::Budding => ReproductionType::Mitosis,
                ReproductionType::Mitosis => ReproductionType::Budding,
            };
            let flip = rng.gen_range(10..15);
            for _ in 0..flip {
                self.tag ^= 1u64 << rng.gen_range(0..u64::BITS);
            }
            return;
        }
        let n = rng.gen_range(0..7);

        for _ in 0..n {
            let roll: f32 = rng.r#gen();

            if roll < 0.60 {
                if self.edges.is_empty() { return; }
                let idx = rng.gen_range(0..self.edges.len());
                let delta = rng.gen_range(-0.3f32..0.3f32);
                self.edges[idx].weight = (self.edges[idx].weight + delta)
                    .clamp(-3f32, 3f32);
                return;
            }

            if roll < 0.75 {
                if self.bias.len() < NUM_INPUTS as usize { return; }
                let idx = rng.gen_range(NUM_INPUTS as usize..self.bias.len());
                let delta = rng.gen_range(-0.2f32..0.2f32);
                self.bias[idx] = (self.bias[idx] + delta).clamp(-2f32, 2f32);
                return;
            }

            self.tag ^= 1u64 << rng.gen_range(0..u64::BITS);

            if roll < 0.85 {
                let n = self.bias.len() as u16;

                let from: u16 = if n > NUM_BASE_NODES && rng.gen_bool(0.4) {
                    rng.gen_range(0..n)
                } else {
                    rng.gen_range(0..NUM_INPUTS)
                };
                let to: u16 = if n > NUM_BASE_NODES && rng.gen_bool(0.3) {
                    rng.gen_range(NUM_INPUTS..n)
                } else {
                    rng.gen_range(NUM_INPUTS..NUM_BASE_NODES)
                };

                if !self.edges.iter().any(|e| e.from == from && e.to == to) {
                    self.edges.push(Edge {
                        from,
                        to,
                        weight: rng.gen_range(-1f32..1f32)
                    });
                    self.dirty = true;
                }
                return;
            }

            if roll < 0.93 {
                if self.edges.is_empty() { return; }
                let idx = rng.gen_range(0..self.edges.len());
                self.edges.swap_remove(idx);
                self.dirty = true;
                return;
            }

            if roll < 0.98 {
                if self.edges.is_empty() { return; }
                let edge_idx = rng.gen_range(0..self.edges.len());
                let e = self.edges[edge_idx];
                let new_node = self.bias.len() as u16;

                self.bias.push(rng.gen_range(-0.5f32..0.5f32));
                self.activation.push(random_activation(rng));
                self.state.push(0.0);
                self.prev_state.push(0.0);

                self.edges[edge_idx] = Edge { from: e.from, to: new_node, weight: 1.0 };
                self.edges.push(Edge { from: new_node, to: e.to, weight: e.weight });
                self.dirty = true;
                return;
            }

            if self.activation.len() > NUM_BASE_NODES as usize {
                let idx = rng.gen_range(NUM_BASE_NODES as usize..self.activation.len());
                self.activation[idx] = random_activation(rng);
            }
        }
    }

    fn rebuild_topology(&mut self) {
        let n = self.bias.len();

        let mut in_degree = vec![0u16; n];
        for e in &self.edges {
            if e.from != e.to {
                in_degree[e.to as usize] += 1;
            }
        }

        let mut queue: Vec<u16> = (0..n as u16)
            .filter(|&i| in_degree[i as usize] == 0)
            .collect();

        self.eval_order.clear();
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head];
            head += 1;
            self.eval_order.push(node);

            for e in &self.edges {
                if e.from == node && e.from != e.to {
                    let t = e.to as usize;
                    in_degree[t] -= 1;
                    if in_degree[t] == 0 {
                        queue.push(e.to);
                    }
                }
            }
        }

        let mut in_eval = vec![false; n];
        for &node in &self.eval_order {
            in_eval[node as usize] = true;
        }
        self.cycle_nodes.clear();
        for i in 0..n as u16 {
            if !in_eval[i as usize] {
                self.cycle_nodes.push(i);
            }
        }
    }
}

#[inline(always)]
fn apply_activation(act: Activation, x: f32) -> f32 {
    match act {
        Activation::Linear => x,
        Activation::Step => if x >= 0.5 { 1.0 } else { 0.0 },
        Activation::Sigmoid => 0.5 + 0.5 * x / (1.0 + x.abs()),
        Activation::Tanh => x / (1.0 + x.abs()),
        //Activation::RadialBasis => 1.0 / (1.0 + x * x),
    }
}

#[inline(always)]
fn is_input(idx: u16) -> bool {
    idx < NUM_INPUTS
}
#[inline(always)]
#[allow(dead_code)]
fn is_output(idx: u16) -> bool {
    idx >= NUM_INPUTS && idx < NUM_BASE_NODES
}
#[inline(always)]
#[allow(dead_code)]
fn is_hidden(idx: u16) -> bool {
    idx >= NUM_BASE_NODES
}

fn random_activation(rng: &mut impl Rng) -> Activation {
    match rng.gen_range(0..20) {
        0..=12 => Activation::Step,
        13..=16 => Activation::Sigmoid,
        17..=18 => Activation::Tanh,
        19 => Activation::Linear,
        _ => Activation::Step,
    }
}
