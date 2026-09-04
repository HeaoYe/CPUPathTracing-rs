mod area_light;
mod image_infinite_light;
mod infinite_light;
mod uniform_infinite_light;

pub use area_light::AreaLight;
pub use image_infinite_light::ImageInfiniteLight;
pub use infinite_light::InfiniteLight;
pub use uniform_infinite_light::UniformInfiniteLight;

use crate::spectrum::SpectrumSample;

pub struct LightSample {
    pub light_point: glam::Vec3,
    pub light_direction: glam::Vec3,
    pub radiance: SpectrumSample,
    pub pdf: SpectrumSample,
}

pub enum Light<'a> {
    Area(AreaLight<'a>),
    Infinite(InfiniteLight<'a>),
}
