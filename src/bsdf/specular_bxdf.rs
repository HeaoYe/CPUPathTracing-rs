use super::bxdf::{Bxdf, ScatteringSample};

pub struct SpecularBxdf {
    albedo: glam::Vec3,
}

impl SpecularBxdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl Bxdf for SpecularBxdf {
    fn sample(
        &self,
        _hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        _rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let light_direction =
            glam::Vec3::new(-view_direction.x, view_direction.y, -view_direction.z);
        let bsdf = self.albedo / light_direction.y.abs();
        Some(ScatteringSample {
            bsdf,
            pdf: 1.0,
            light_direction,
        })
    }
}
