use super::ScatteringSample;

pub struct SpecularBsdf {
    albedo: glam::Vec3,
}

impl SpecularBsdf {
    pub fn new(albedo: glam::Vec3) -> Self {
        Self { albedo }
    }
}

impl SpecularBsdf {
    pub(super) fn is_delta_distribution(&self) -> bool {
        true
    }

    pub(super) fn sample(&self, view_direction: glam::Vec3) -> Option<ScatteringSample> {
        let light_direction = glam::vec3(-view_direction.x, view_direction.y, -view_direction.z);
        let bsdf = self.albedo / light_direction.y.abs();
        Some(ScatteringSample::new(bsdf, 1.0, light_direction))
    }

    pub(super) fn bsdf(&self) -> glam::Vec3 {
        glam::Vec3::ZERO
    }

    pub(super) fn pdf(&self) -> f32 {
        0.0
    }
}
