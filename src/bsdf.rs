mod conductor_bsdf;
mod dielectric_bsdf;
mod diffuse_bsdf;
mod ground_bsdf;
mod microfacet_theory;
mod specular_bsdf;

pub use conductor_bsdf::ConductorBsdf;
pub use dielectric_bsdf::DielectricBsdf;
pub use diffuse_bsdf::DiffuseBsdf;
pub use ground_bsdf::GroundBsdf;
pub use specular_bsdf::SpecularBsdf;

use crate::spectrum::{SpectrumSample, WavelengthSample};

pub struct ScatteringSample {
    pub bsdf: SpectrumSample,
    pub pdf: SpectrumSample,
    pub light_direction: glam::Vec3,
    pub eta_scale: SpectrumSample,
    pub dispersive_refraction: bool,
}

impl ScatteringSample {
    pub fn new(bsdf: SpectrumSample, pdf: SpectrumSample, light_direction: glam::Vec3) -> Self {
        Self {
            bsdf,
            pdf,
            light_direction,
            eta_scale: SpectrumSample::ONE,
            dispersive_refraction: false,
        }
    }

    pub fn with_eta_scale(mut self, eta_scale: SpectrumSample) -> Self {
        self.eta_scale = eta_scale;
        self
    }

    pub fn with_dispersive_refraction(mut self) -> Self {
        self.dispersive_refraction = true;
        self
    }
}

#[derive(Clone, Copy)]
pub enum Bsdf<'a> {
    Conductor(ConductorBsdf<'a>),
    Dielectric(DielectricBsdf<'a>),
    Diffuse(DiffuseBsdf<'a>),
    Ground(GroundBsdf<'a>),
    Specular(SpecularBsdf<'a>),
}

impl Bsdf<'_> {
    pub fn is_delta_distribution(&self) -> bool {
        match self {
            Self::Conductor(bxdf) => bxdf.is_delta_distribution(),
            Self::Dielectric(bxdf) => bxdf.is_delta_distribution(),
            Self::Diffuse(bxdf) => bxdf.is_delta_distribution(),
            Self::Ground(bxdf) => bxdf.is_delta_distribution(),
            Self::Specular(bxdf) => bxdf.is_delta_distribution(),
        }
    }

    pub fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
    ) -> Option<ScatteringSample> {
        let scattering_sample = match self {
            Self::Conductor(bxdf) => bxdf.sample(view_direction, rng, wavelength),
            Self::Dielectric(bxdf) => bxdf.sample(view_direction, rng, wavelength),
            Self::Diffuse(bxdf) => bxdf.sample(view_direction, rng, wavelength),
            Self::Ground(bxdf) => bxdf.sample(hit_point, view_direction, rng, wavelength),
            Self::Specular(bxdf) => bxdf.sample(view_direction, wavelength),
        }?;

        if scattering_sample.bsdf == SpectrumSample::ZERO
            || scattering_sample.pdf[0] == 0.0
            || scattering_sample.light_direction.y == 0.0
        {
            None
        } else {
            Some(scattering_sample)
        }
    }

    pub fn bsdf(
        &self,
        hit_point: glam::Vec3,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        match self {
            Self::Conductor(bxdf) => bxdf.bsdf(light_direction, view_direction, wavelength),
            Self::Dielectric(bxdf) => bxdf.bsdf(light_direction, view_direction, wavelength),
            Self::Diffuse(bxdf) => bxdf.bsdf(light_direction, view_direction, wavelength),
            Self::Ground(bxdf) => bxdf.bsdf(hit_point, light_direction, view_direction, wavelength),
            Self::Specular(bxdf) => bxdf.bsdf(),
        }
    }

    pub fn pdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        match self {
            Self::Conductor(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Dielectric(bxdf) => bxdf.pdf(light_direction, view_direction, wavelength),
            Self::Diffuse(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Ground(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Specular(bxdf) => bxdf.pdf(),
        }
    }
}
