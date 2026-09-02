mod bxdf;
mod diffuse_bxdf;
mod specular_bxdf;

pub use bxdf::{Bxdf, ScatteringSample};
pub use diffuse_bxdf::DiffuseBxdf;
pub use specular_bxdf::SpecularBxdf;

pub enum Bsdf {
    Diffuse(DiffuseBxdf),
    Specular(SpecularBxdf),
}

impl Bxdf for Bsdf {
    fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let scattering_sample = match self {
            Bsdf::Diffuse(bxdf) => bxdf.sample(view_direction, rng),
            Bsdf::Specular(bxdf) => bxdf.sample(view_direction, rng),
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
