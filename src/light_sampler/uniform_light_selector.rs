use super::{LightSelection, LightSelector, MisCompensation};
use crate::{
    light::Light,
    scene::{LightId, Scene},
    util::Rng,
};

pub struct UniformLightSelector {
    light_count: usize,
    active_lights: Vec<LightId>,
}

impl LightSelector for UniformLightSelector {
    fn new(scene: &Scene, mis_compensation: MisCompensation) -> Self {
        let mut active_lights = Vec::new();
        for (light_id, light) in scene.lights() {
            match light {
                Light::Area(_) => active_lights.push(light_id),
                Light::Infinite(light) => match mis_compensation {
                    MisCompensation::Disabled => active_lights.push(light_id),
                    MisCompensation::Enabled => {
                        if !light.skip_mis_compensation() {
                            active_lights.push(light_id);
                        }
                    }
                },
            }
        }
        Self {
            light_count: active_lights.len(),
            active_lights,
        }
    }

    fn sample_light_source(&self, rng: &mut Rng) -> Option<LightSelection> {
        if self.light_count == 0 {
            return None;
        }
        Some(LightSelection {
            id: self.active_lights[rng.uniform_range(0..self.light_count)],
            pmf: 1.0 / self.light_count as f32,
        })
    }

    fn pmf(&self, light_id: LightId) -> f32 {
        if self.active_lights.binary_search(&light_id).is_ok() {
            1.0 / self.light_count as f32
        } else {
            0.0
        }
    }
}
