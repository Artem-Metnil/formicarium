use rand::Rng;

#[derive(Clone)]
enum NeuronType {
    Basic,
    RadialBasis,
    Output,
}

impl NeuronType {
    #[inline(always)]
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            NeuronType::Basic => if x >= 0.5 { 1.0 } else { 0.0 },
            NeuronType::RadialBasis => if x > 0.45 && x < 0.55 { 1.0 } else { 0.0 },
            NeuronType::Output => if x >= 0.5 { 1.0 } else { 0.0 },
        }
    }
}

#[derive(Clone)]
pub struct Brain {
    pub tag: usize,

    w1: [[f32; 7]; 8],
    b1: [f32; 8],
    neuron_type_w1: [NeuronType; 8],

    w2: [[f32; 8]; 8],
    b2: [f32; 8],
    neuron_type_w2: [NeuronType; 8],

    w3: [[f32; 8]; 8],
    b3: [f32; 8],
    neuron_type_w3: [NeuronType; 8],

    w4: [[f32; 8]; 8],
    b4: [f32; 8],
    neuron_type_w4: [NeuronType; 8],

    w5: [[f32; 8]; 6],
    b5: [f32; 6],
    neuron_type_w5: [NeuronType; 6],
}

impl Brain {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let random_neuron_type = |rng: &mut rand::rngs::ThreadRng| {
            match rng.gen_range(0..10) {
                1..=2 => NeuronType::RadialBasis,
                _ => NeuronType::Basic,
            }
        };

        fn gen_weights<const R: usize, const C: usize>(rng: &mut impl Rng) -> [[f32; C]; R] {
            std::array::from_fn(|_| std::array::from_fn(|_| {
                if rng.gen_bool(0.5) { rng.gen_range(-1.0..1.0) } else { 0.0f32 }
            }))
        }

        Self {
            tag: rng.gen_range(0..=usize::MAX),

            w1: gen_weights(&mut rng),
            b1: std::array::from_fn(|_| rng.gen_range(-0.5..0.5)),
            neuron_type_w1: std::array::from_fn(|_| random_neuron_type(&mut rng)),

            w2: gen_weights(&mut rng),
            b2: std::array::from_fn(|_| rng.gen_range(-0.5..0.5)),
            neuron_type_w2: std::array::from_fn(|_| random_neuron_type(&mut rng)),

            w3: gen_weights(&mut rng),
            b3: std::array::from_fn(|_| rng.gen_range(-0.5..0.5)),
            neuron_type_w3: std::array::from_fn(|_| random_neuron_type(&mut rng)),

            w4: gen_weights(&mut rng),
            b4: std::array::from_fn(|_| rng.gen_range(-0.5..0.5)),
            neuron_type_w4: std::array::from_fn(|_| random_neuron_type(&mut rng)),

            w5: gen_weights(&mut rng),
            b5: std::array::from_fn(|_| rng.gen_range(-0.5..0.5)),
            neuron_type_w5: std::array::from_fn(|_| NeuronType::Output),
        }
    }

    pub fn forward(&self, inputs: &[f32; 7]) -> [f32; 6] {
        let mut h1 = [0.0f32; 8];
        for i in 0..8 {
            let mut sum = self.b1[i];
            for j in 0..7 {
                sum += inputs[j] * self.w1[i][j];
            }
            h1[i] = self.neuron_type_w1[i].apply(sum);
        }

        let mut h2 = [0.0f32; 8];
        for i in 0..8 {
            let mut sum = self.b2[i];
            for j in 0..8 {
                sum += h1[j] * self.w2[i][j];
            }
            h2[i] = self.neuron_type_w2[i].apply(sum);
        }

        let mut h3 = [0.0f32; 8];
        for i in 0..8 {
            let mut sum = self.b3[i];
            for j in 0..8 {
                sum += h2[j] * self.w3[i][j];
            }
            h3[i] = self.neuron_type_w3[i].apply(sum);
        }

        let mut h4 = [0.0f32; 8];
        for i in 0..8 {
            let mut sum = self.b4[i];
            for j in 0..8 {
                sum += h3[j] * self.w4[i][j];
            }
            h4[i] = self.neuron_type_w4[i].apply(sum);
        }

        let mut outputs = [0.0f32; 6];
        for i in 0..6 {
            let mut sum = self.b5[i];
            for j in 0..8 {
                sum += h4[j] * self.w5[i][j];
            }
            outputs[i] = self.neuron_type_w5[i].apply(sum);
        }
        outputs
    }

    pub fn is_relative(&self, tag : usize) -> bool {
        let diff = self.tag ^ tag;
        diff.count_ones() < 10
    }
}