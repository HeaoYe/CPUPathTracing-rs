use rand::RngExt;
use rand_pcg::Pcg32;

pub struct Rng {
    pcg: Pcg32,
}

impl Rng {
    pub fn new(state: u64, stream: u64) -> Self {
        Self {
            pcg: Pcg32::new(state, stream),
        }
    }

    pub fn uniform(&mut self) -> f32 {
        self.pcg.random()
    }
}
