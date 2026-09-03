use super::LightSample;
use crate::sample::uniform;

pub struct UniformInfiniteLight {
    radiance: glam::Vec3,
}

impl UniformInfiniteLight {
    pub fn new(radiance: glam::Vec3) -> Self {
        Self { radiance }
    }

    pub fn power(&self, scene_radius: f32) -> f32 {
        4.0 * std::f32::consts::PI
            * std::f32::consts::PI
            * scene_radius
            * scene_radius
            * self.radiance.max_element()
    }

    pub fn sample(
        &self,
        surface_point: glam::Vec3,
        scene_radius: f32,
        rng: &mut crate::util::Rng,
    ) -> Option<LightSample> {
        let light_direction = uniform::sphere(rng.uniform(), rng.uniform());
        Some(LightSample {
            light_point: surface_point + 2.0 * scene_radius * light_direction,
            light_direction,
            radiance: self.radiance,
            pdf: 0.25 * std::f32::consts::FRAC_1_PI,
        })
    }

    pub fn radiance(&self) -> glam::Vec3 {
        self.radiance
    }
}
