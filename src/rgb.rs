use std::ops::{Div, Mul};

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_array(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<glam::Vec3> for RGB {
    fn from(value: glam::Vec3) -> Self {
        let rgb = value
            .powf(1.0 / 2.2)
            .mul(255.0)
            .clamp(glam::Vec3::ZERO, glam::Vec3::splat(255.0))
            .as_u8vec3();
        Self {
            r: rgb.x,
            g: rgb.y,
            b: rgb.z,
        }
    }
}

impl From<RGB> for glam::Vec3 {
    fn from(value: RGB) -> Self {
        glam::U8Vec3::new(value.r, value.g, value.b)
            .as_vec3()
            .div(255.0)
            .powf(2.2)
    }
}
