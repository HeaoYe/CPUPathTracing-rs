use super::{LightSample, UniformInfiniteLight};

pub enum InfiniteLight {
    Uniform(UniformInfiniteLight),
}

impl InfiniteLight {
    pub(crate) fn power(&self, scene_radius: f32) -> f32 {
        match self {
            Self::Uniform(light) => light.power(scene_radius),
        }
    }

    pub(crate) fn sample(
        &self,
        surface_point: glam::Vec3,
        scene_radius: f32,
        rng: &mut crate::util::Rng,
    ) -> Option<LightSample> {
        match self {
            Self::Uniform(light) => light.sample(surface_point, scene_radius, rng),
        }
    }

    pub(crate) fn radiance(&self) -> glam::Vec3 {
        match self {
            Self::Uniform(light) => light.radiance(),
        }
    }

    pub(crate) fn pdf(&self) -> f32 {
        match self {
            Self::Uniform(light) => light.pdf(),
        }
    }
}
