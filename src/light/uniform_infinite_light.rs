use super::LightSample;
use crate::{
    light_sampler::MisCompensation,
    sample::uniform,
    spectrum::{Spectrum, SpectrumSample, WavelengthSample},
};

pub struct UniformInfiniteLight<'a> {
    radiance: &'a Spectrum<'a>,
}

impl<'a> UniformInfiniteLight<'a> {
    pub(crate) fn new(radiance: &'a Spectrum<'a>) -> Self {
        Self { radiance }
    }

    pub(crate) fn power(&self, scene_radius: f32) -> f32 {
        4.0 * std::f32::consts::PI
            * std::f32::consts::PI
            * scene_radius
            * scene_radius
            * self.radiance.max()
    }

    pub(crate) fn skip_mis_compensation(&self) -> bool {
        true
    }

    pub(crate) fn sample(
        &self,
        surface_point: glam::Vec3,
        scene_radius: f32,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
        mis_compensation: MisCompensation,
    ) -> Option<LightSample> {
        match mis_compensation {
            MisCompensation::Disabled => {
                let light_direction = uniform::sphere(rng.uniform(), rng.uniform());
                Some(LightSample {
                    light_point: surface_point + 2.0 * scene_radius * light_direction,
                    light_direction,
                    radiance: self.radiance.sample(wavelength),
                    pdf: SpectrumSample::splat(0.25 * std::f32::consts::FRAC_1_PI),
                })
            }
            MisCompensation::Enabled => None,
        }
    }

    pub(crate) fn radiance(&self, wavelength: &WavelengthSample) -> SpectrumSample {
        self.radiance.sample(wavelength)
    }

    pub(crate) fn pdf(&self, mis_compensation: MisCompensation) -> SpectrumSample {
        match mis_compensation {
            MisCompensation::Disabled => SpectrumSample::splat(0.25 * std::f32::consts::FRAC_1_PI),
            MisCompensation::Enabled => SpectrumSample::ZERO,
        }
    }
}
