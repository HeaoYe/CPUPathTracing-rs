use super::ScatteringSample;
use crate::{
    sample::importance,
    spectrum::{Spectrum, SpectrumSample, WavelengthSample},
};

#[derive(Clone, Copy)]
pub struct GroundBsdf<'a> {
    albedo: &'a Spectrum<'a>,
}

impl<'a> GroundBsdf<'a> {
    pub fn new(albedo: &'a Spectrum<'a>) -> Self {
        Self { albedo }
    }
}

impl GroundBsdf<'_> {
    pub(super) fn is_delta_distribution(&self) -> bool {
        false
    }

    pub(super) fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let bsdf = self.bsdf(hit_point, light_direction, view_direction, wavelength);
        Some(ScatteringSample::new(
            bsdf,
            pdf,
            light_direction * view_direction.y.signum(),
        ))
    }

    pub(super) fn bsdf(
        &self,
        hit_point: glam::Vec3,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        if light_direction.y * view_direction.y <= 0.0 {
            SpectrumSample::ZERO
        } else {
            let mut bsdf = self.albedo.sample(wavelength) / std::f32::consts::PI;
            if ((hit_point.x * 8.0 + 0.5).floor() as i32) % 8 == 0
                || ((hit_point.z * 8.0 + 0.5).floor() as i32) % 8 == 0
            {
                bsdf *= 0.1;
            }
            bsdf
        }
    }

    pub(super) fn pdf(&self, light_direction: glam::Vec3, view_direction: glam::Vec3) -> f32 {
        if light_direction.y * view_direction.y <= 0.0 {
            0.0
        } else {
            importance::cosine_hemisphere_pdf(light_direction)
        }
    }
}
