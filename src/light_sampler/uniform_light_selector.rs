use super::{LightSelection, LightSelector};
use crate::{
    scene::{LightId, Scene},
    util::Rng,
};

pub struct UniformLightSelector {
    light_count: usize,
}

impl LightSelector for UniformLightSelector {
    fn new(scene: &Scene) -> Self {
        Self {
            light_count: scene.lights().len(),
        }
    }

    fn sample_light_source(&self, rng: &mut Rng) -> Option<LightSelection> {
        if self.light_count == 0 {
            return None;
        }
        Some(LightSelection {
            id: LightId(rng.uniform_range(0..self.light_count)),
            pmf: 1.0 / self.light_count as f32,
        })
    }

    fn pmf(&self, _light_source: LightId) -> f32 {
        1.0 / self.light_count as f32
    }
}
