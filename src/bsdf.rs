mod conductor_bsdf;
mod dielectric_bsdf;
mod diffuse_bsdf;
mod ground_bsdf;
mod microfacet_theory;
mod specular_bsdf;

pub struct ScatteringSample {
    pub bsdf: glam::Vec3,
    pub pdf: f32,
    pub light_direction: glam::Vec3,
    pub eta_scale: f32,
}

impl ScatteringSample {
    pub fn new(bsdf: glam::Vec3, pdf: f32, light_direction: glam::Vec3) -> Self {
        Self {
            bsdf,
            pdf,
            light_direction,
            eta_scale: 1.0,
        }
    }

    pub fn new_with_eta_scale(
        bsdf: glam::Vec3,
        pdf: f32,
        light_direction: glam::Vec3,
        eta_scale: f32,
    ) -> Self {
        Self {
            bsdf,
            pdf,
            light_direction,
            eta_scale,
        }
    }
}

pub use conductor_bsdf::ConductorBsdf;
pub use dielectric_bsdf::DielectricBsdf;
pub use diffuse_bsdf::DiffuseBsdf;
pub use ground_bsdf::GroundBsdf;
pub use specular_bsdf::SpecularBsdf;

pub enum Bsdf {
    Conductor(ConductorBsdf),
    Dielectric(DielectricBsdf),
    Diffuse(DiffuseBsdf),
    Ground(GroundBsdf),
    Specular(SpecularBsdf),
}

impl Bsdf {
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
    ) -> Option<ScatteringSample> {
        let scattering_sample = match self {
            Self::Conductor(bxdf) => bxdf.sample(view_direction, rng),
            Self::Dielectric(bxdf) => bxdf.sample(view_direction, rng),
            Self::Diffuse(bxdf) => bxdf.sample(view_direction, rng),
            Self::Ground(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Self::Specular(bxdf) => bxdf.sample(view_direction),
        }?;

        if scattering_sample.bsdf == glam::Vec3::ZERO
            || scattering_sample.pdf == 0.0
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
    ) -> glam::Vec3 {
        match self {
            Self::Conductor(bxdf) => bxdf.bsdf(light_direction, view_direction),
            Self::Dielectric(bxdf) => bxdf.bsdf(light_direction, view_direction),
            Self::Diffuse(bxdf) => bxdf.bsdf(light_direction, view_direction),
            Self::Ground(bxdf) => bxdf.bsdf(hit_point, light_direction, view_direction),
            Self::Specular(bxdf) => bxdf.bsdf(),
        }
    }

    pub fn pdf(&self, light_direction: glam::Vec3, view_direction: glam::Vec3) -> f32 {
        match self {
            Self::Conductor(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Dielectric(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Diffuse(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Ground(bxdf) => bxdf.pdf(light_direction, view_direction),
            Self::Specular(bxdf) => bxdf.pdf(),
        }
    }
}
