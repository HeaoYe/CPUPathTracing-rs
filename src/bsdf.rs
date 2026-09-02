mod bxdf;
mod conductor_bxdf;
mod dielectric_bxdf;
mod diffuse_bxdf;
mod ground_bxdf;
mod microfacet_theory;
mod specular_bxdf;

pub use bxdf::{Bxdf, ScatteringSample};
pub use conductor_bxdf::ConductorBxdf;
pub use dielectric_bxdf::DielectricBxdf;
pub use diffuse_bxdf::DiffuseBxdf;
pub use ground_bxdf::GroundBxdf;
pub use specular_bxdf::SpecularBxdf;

pub enum Bsdf {
    Conductor(ConductorBxdf),
    Dielectric(DielectricBxdf),
    Diffuse(DiffuseBxdf),
    Ground(GroundBxdf),
    Specular(SpecularBxdf),
}

impl Bxdf for Bsdf {
    fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let scattering_sample = match self {
            Bsdf::Conductor(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Bsdf::Dielectric(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Bsdf::Diffuse(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Bsdf::Ground(bxdf) => bxdf.sample(hit_point, view_direction, rng),
            Bsdf::Specular(bxdf) => bxdf.sample(hit_point, view_direction, rng),
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
}
