use super::ScatteringSample;
use crate::sample::importance;

pub struct DiffuseBsdf {
    albedo: glam::Vec3,
}

impl DiffuseBsdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl DiffuseBsdf {
    pub(super) fn is_delta_distribution(&self) -> bool {
        false
    }

    pub(super) fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let bsdf = self.albedo / std::f32::consts::PI;
        Some(ScatteringSample::new(
            bsdf,
            pdf,
            light_direction * view_direction.y.signum(),
        ))
    }

    pub(super) fn bsdf(&self) -> glam::Vec3 {
        self.albedo / std::f32::consts::PI
    }

    pub(super) fn pdf(&self, light_direction: glam::Vec3, view_direction: glam::Vec3) -> f32 {
        if light_direction.y * view_direction.y <= 0.0 {
            0.0
        } else {
            importance::cosine_hemisphere_pdf(light_direction)
        }
    }
}
