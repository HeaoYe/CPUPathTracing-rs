use super::ScatteringSample;
use crate::sample::importance;

pub struct GroundBsdf {
    albedo: glam::Vec3,
}

impl GroundBsdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl GroundBsdf {
    pub(super) fn is_delta_distribution(&self) -> bool {
        false
    }

    pub(super) fn sample(
        &self,
        hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        let light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
        let pdf = importance::cosine_hemisphere_pdf(light_direction);
        let bsdf = self.bsdf(hit_point);
        Some(ScatteringSample::new(
            bsdf,
            pdf,
            light_direction * view_direction.y.signum(),
        ))
    }

    pub(super) fn bsdf(&self, hit_point: glam::Vec3) -> glam::Vec3 {
        let mut bsdf = self.albedo / std::f32::consts::PI;
        if ((hit_point.x * 8.0 + 0.5).floor() as i32) % 8 == 0
            || ((hit_point.z * 8.0 + 0.5).floor() as i32) % 8 == 0
        {
            bsdf *= 0.1;
        }
        bsdf
    }

    pub(super) fn pdf(&self, light_direction: glam::Vec3, view_direction: glam::Vec3) -> f32 {
        if light_direction.y * view_direction.y <= 0.0 {
            0.0
        } else {
            importance::cosine_hemisphere_pdf(light_direction)
        }
    }
}
