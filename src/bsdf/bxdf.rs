use crate::util::Rng;

pub struct ScatteringSample {
    pub bsdf: glam::Vec3,
    pub pdf: f32,
    pub light_direction: glam::Vec3,
}

pub trait Bxdf {
    fn sample(&self, view_direction: glam::Vec3, rng: &mut Rng) -> Option<ScatteringSample>;
}
