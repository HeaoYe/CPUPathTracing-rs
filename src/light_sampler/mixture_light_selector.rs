use super::{LightSelection, LightSelector, MisCompensation};
use crate::{
    scene::{LightId, Scene},
    util::Rng,
};

pub struct MixtureLightSelector<const PERCENT_A: u32, A, B> {
    weight_a: f32,
    a: A,
    b: B,
}

impl<const PERCENT_A: u32, A, B> LightSelector for MixtureLightSelector<PERCENT_A, A, B>
where
    A: LightSelector,
    B: LightSelector,
{
    fn new(scene: &Scene, mis_compensation: MisCompensation) -> Self {
        Self {
            weight_a: PERCENT_A.min(100) as f32 / 100.0,
            a: A::new(scene, mis_compensation),
            b: B::new(scene, mis_compensation),
        }
    }

    fn sample_light_source(&self, rng: &mut Rng) -> Option<LightSelection> {
        let id = if rng.uniform() < self.weight_a {
            self.a.sample_light_source(rng)?.id
        } else {
            self.b.sample_light_source(rng)?.id
        };
        Some(LightSelection {
            id,
            pmf: self.pmf(id),
        })
    }

    fn pmf(&self, light_source: LightId) -> f32 {
        self.weight_a * self.a.pmf(light_source) + (1.0 - self.weight_a) * self.b.pmf(light_source)
    }
}
