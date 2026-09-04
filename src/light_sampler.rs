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
    scene::{LightId, Scene},
    spectrum::{SpectrumSample, WavelengthSample},
    util::Rng,
};

#[derive(Clone, Copy)]
pub enum MisCompensation {
    Disabled,
    Enabled,
}

pub struct LightSampler<'a, L> {
    scene: &'a Scene<'a>,
    light_selector: L,
    mis_compensation: MisCompensation,
}

impl<L> LightSampler<'_, L> {
    pub fn mis_compensation(&self) -> MisCompensation {
        self.mis_compensation
    }
}

impl<'a, L: LightSelector> LightSampler<'a, L> {
    pub fn new(scene: &'a Scene<'a>, mis_compensation: MisCompensation) -> Self {
        Self {
            scene,
            light_selector: L::new(scene, mis_compensation),
            mis_compensation,
        }
    }
}

impl<L: LightSelector> LightSampler<'_, L> {
    pub fn sample_light(
        &self,
        surface_point: glam::Vec3,
        rng: &mut Rng,
        wavelength: &WavelengthSample,
    ) -> Option<LightSample> {
        let light_selection = self.light_selector.sample_light_source(rng)?;
        let light = &self.scene.get_light(light_selection.id)?;
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
                    wavelength,
                )
            }
            Light::Infinite(light) => light.sample(
                surface_point,
                self.scene.radius(),
                rng,
                wavelength,
                self.mis_compensation,
            ),
        }?;
        sample.pdf *= light_selection.pmf;
        Some(sample)
    }

    pub fn pdf(
        &self,
        light_id: LightId,
        surface_point: glam::Vec3,
        light_point: glam::Vec3,
        normal: glam::Vec3,
    ) -> SpectrumSample {
        let light = &self.scene.get_light(light_id).unwrap();
        let pdf = match light {
            Light::Area(light) => {
                let shape_instance = self
                    .scene
                    .get_shape_instance(light.shape_instance_id)
                    .unwrap();
                light.pdf(shape_instance.shape(), surface_point, light_point, normal)
            }
            Light::Infinite(light) => light.pdf(
                (light_point - surface_point).normalize(),
                self.mis_compensation,
            ),
        };
        SpectrumSample::splat(self.light_selector.pmf(light_id)) * pdf
    }
}
