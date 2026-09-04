use super::ScatteringSample;
use crate::spectrum::{Spectrum, SpectrumSample, WavelengthSample};

#[derive(Clone, Copy)]
pub struct SpecularBsdf<'a> {
    albedo: &'a Spectrum<'a>,
}

impl<'a> SpecularBsdf<'a> {
    pub fn new(albedo: &'a Spectrum<'a>) -> Self {
        Self { albedo }
    }
}

impl SpecularBsdf<'_> {
    pub(super) fn is_delta_distribution(&self) -> bool {
        true
    }

    pub(super) fn sample(
        &self,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> Option<ScatteringSample> {
        let light_direction = glam::vec3(-view_direction.x, view_direction.y, -view_direction.z);
        let bsdf = self.albedo.sample(wavelength) / light_direction.y.abs();
        Some(ScatteringSample::new(bsdf, 1.0, light_direction))
    }

    pub(super) fn bsdf(&self) -> SpectrumSample {
        SpectrumSample::ZERO
    }

    pub(super) fn pdf(&self) -> f32 {
        0.0
    }
}
