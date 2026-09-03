mod light_selector;
mod mixture_light_selector;
mod power_light_selector;
mod uniform_light_selector;

pub use light_selector::{LightSelection, LightSelector};
pub use mixture_light_selector::MixtureLightSelector;
pub use power_light_selector::PowerLightSelector;
pub use uniform_light_selector::UniformLightSelector;

use crate::{
    light::{Light, LightSample},
    scene::Scene,
    util::Rng,
};

pub struct LightSampler<'a, L> {
    scene: &'a Scene<'a>,
    light_selector: L,
}

impl<'a, L: LightSelector> LightSampler<'a, L> {
    pub fn new(scene: &'a Scene<'a>) -> Self {
        Self {
            scene,
            light_selector: L::new(scene),
        }
    }
}

impl<L: LightSelector> LightSampler<'_, L> {
    pub fn sample_light(&self, surface_point: glam::Vec3, rng: &mut Rng) -> Option<LightSample> {
        let light_selection = self.light_selector.sample_light_source(rng)?;
        let light = &self.scene.lights()[light_selection.id.0];
        let mut sample = match light {
            Light::Area(light) => {
                let shape_instance = self
                    .scene
                    .get_shape_instance(light.shape_instance_id)
                    .unwrap();
                light.sample(
                    surface_point,
                    shape_instance.shape(),
                    &shape_instance.world_from_object(),
                    &shape_instance.object_from_world(),
                    rng,
                )
            }
            Light::UniformInfinite(light) => light.sample(surface_point, self.scene.radius(), rng),
        }?;
        sample.pdf *= light_selection.pmf;
        Some(sample)
    }
}
