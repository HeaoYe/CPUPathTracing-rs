use super::{ImageInfiniteLight, LightSample, UniformInfiniteLight};
use crate::{
    light_sampler::MisCompensation,
    spectrum::{SpectrumSample, WavelengthSample},
};

pub enum InfiniteLight<'a> {
    Uniform(UniformInfiniteLight<'a>),
    Image(Box<ImageInfiniteLight<'a>>),
}

impl<'a> InfiniteLight<'a> {
    pub(crate) fn power(&self, scene_radius: f32) -> f32 {
        match self {
            Self::Uniform(light) => light.power(scene_radius),
            Self::Image(light) => light.power(scene_radius),
        }
    }

    pub(crate) fn skip_mis_compensation(&self) -> bool {
        match self {
            Self::Uniform(light) => light.skip_mis_compensation(),
            Self::Image(light) => light.skip_mis_compensation(),
        }
    }

    pub(crate) fn sample(
        &self,
        surface_point: glam::Vec3,
        scene_radius: f32,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
        mis_compensation: MisCompensation,
    ) -> Option<LightSample> {
        match self {
            Self::Uniform(light) => light.sample(
                surface_point,
                scene_radius,
                rng,
                wavelength,
                mis_compensation,
            ),
            Self::Image(light) => light.sample(
                surface_point,
                scene_radius,
                rng,
                wavelength,
                mis_compensation,
            ),
        }
    }

    pub(crate) fn radiance(
        &self,
        light_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        match self {
            Self::Uniform(light) => light.radiance(wavelength),
            Self::Image(light) => light.radiance(light_direction, wavelength),
        }
    }

    pub(crate) fn pdf(
        &self,
        light_direction: glam::Vec3,
        mis_compensation: MisCompensation,
    ) -> SpectrumSample {
        match self {
            Self::Uniform(light) => light.pdf(mis_compensation),
            Self::Image(light) => light.pdf(light_direction, mis_compensation),
        }
    }
}
