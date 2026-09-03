mod area_light;
mod infinite_light;
mod uniform_infinite_light;

pub use area_light::AreaLight;
pub use infinite_light::InfiniteLight;
pub use uniform_infinite_light::UniformInfiniteLight;

pub struct LightSample {
    pub light_point: glam::Vec3,
    pub light_direction: glam::Vec3,
    pub radiance: glam::Vec3,
    pub pdf: f32,
}

pub enum Light {
    Area(AreaLight),
    Infinite(InfiniteLight),
}
