use super::bxdf::{Bxdf, ScatteringSample};
use crate::sample::importance;

pub struct GroundBxdf {
    albedo: glam::Vec3,
}

impl GroundBxdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl Bxdf for GroundBxdf {
    fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let mut bsdf = self.albedo / std::f32::consts::PI;
        if ((hit_point.x * 8.0 + 0.5).floor() as i32) % 8 == 0
            || ((hit_point.z * 8.0 + 0.5).floor() as i32) % 8 == 0
        {
            bsdf *= 0.1;
        }
        Some(ScatteringSample {
            bsdf,
            pdf,
            light_direction: light_direction * view_direction.y.signum(),
        })
    }
}
