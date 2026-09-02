use super::bxdf::{Bxdf, ScatteringSample};
use crate::sample::importance;

pub struct DiffuseBxdf {
    albedo: glam::Vec3,
}

impl DiffuseBxdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl Bxdf for DiffuseBxdf {
    fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let bsdf = self.albedo / std::f32::consts::PI;
        Some(ScatteringSample {
            bsdf,
            pdf,
            light_direction: light_direction * view_direction.y.signum(),
        })
    }
}
