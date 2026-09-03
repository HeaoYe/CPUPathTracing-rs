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
            Bsdf::Conductor(bxdf) => bxdf.is_delta_distribution(),
            Bsdf::Dielectric(bxdf) => bxdf.is_delta_distribution(),
            Bsdf::Diffuse(bxdf) => bxdf.is_delta_distribution(),
            Bsdf::Ground(bxdf) => bxdf.is_delta_distribution(),
            Bsdf::Specular(bxdf) => bxdf.is_delta_distribution(),
        }
    }

    pub fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let scattering_sample = match self {
            Bsdf::Conductor(bxdf) => bxdf.sample(view_direction, rng),
            Bsdf::Dielectric(bxdf) => bxdf.sample(view_direction, rng),
            Bsdf::Diffuse(bxdf) => bxdf.sample(view_direction, rng),
            Bsdf::Ground(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Bsdf::Specular(bxdf) => bxdf.sample(view_direction),
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
            Bsdf::Conductor(bxdf) => bxdf.bsdf(light_direction, view_direction),
            Bsdf::Dielectric(bxdf) => bxdf.bsdf(light_direction, view_direction),
            Bsdf::Diffuse(bxdf) => bxdf.bsdf(),
            Bsdf::Ground(bxdf) => bxdf.bsdf(hit_point),
            Bsdf::Specular(bxdf) => bxdf.bsdf(),
        }
    }
}
