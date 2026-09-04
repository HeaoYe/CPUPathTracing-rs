use super::ScatteringSample;
use crate::{
    sample::importance,
    spectrum::{Spectrum, SpectrumSample, WavelengthSample},
};

#[derive(Clone, Copy)]
pub struct DiffuseBsdf<'a> {
    albedo: &'a Spectrum<'a>,
}

impl<'a> DiffuseBsdf<'a> {
    pub fn new(albedo: &'a Spectrum<'a>) -> Self {
        Self { albedo }
    }
}

impl DiffuseBsdf<'_> {
    pub(super) fn is_delta_distribution(&self) -> bool {
        false
    }

    pub(super) fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let bsdf = self.albedo.sample(wavelength) / std::f32::consts::PI;
        Some(ScatteringSample::new(
            bsdf,
            SpectrumSample::splat(pdf),
            light_direction * view_direction.y.signum(),
        ))
    }

    pub(super) fn bsdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        if light_direction.y * view_direction.y <= 0.0 {
            SpectrumSample::ZERO
        } else {
            self.albedo.sample(wavelength) / std::f32::consts::PI
        }
    }

    pub(super) fn pdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
    ) -> SpectrumSample {
        if light_direction.y * view_direction.y <= 0.0 {
            SpectrumSample::ZERO
        } else {
            SpectrumSample::splat(importance::cosine_hemisphere_pdf(light_direction))
        }
    }
}
