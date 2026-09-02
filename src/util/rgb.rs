use std::ops::{Div, Mul};

pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_array(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<glam::Vec3> for Rgb {
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

impl From<glam::U8Vec3> for Rgb {
    fn from(value: glam::U8Vec3) -> Self {
        Self {
            r: value.x,
            g: value.y,
            b: value.z,
        }
    }
}

impl From<Rgb> for glam::Vec3 {
    fn from(value: Rgb) -> Self {
        glam::u8vec3(value.r, value.g, value.b)
            .as_vec3()
            .div(255.0)
            .powf(2.2)
    }
}

impl From<Rgb> for glam::U8Vec3 {
    fn from(value: Rgb) -> Self {
        glam::u8vec3(value.r, value.g, value.b)
    }
}

impl Rgb {
    pub fn generate_heatmap_rgb(t: f32) -> Rgb {
        let color_pallet: [Rgb; 25] = [
            Rgb::new(68, 1, 84),
            Rgb::new(71, 17, 100),
            Rgb::new(72, 31, 112),
            Rgb::new(71, 45, 123),
            Rgb::new(68, 58, 131),
            Rgb::new(64, 70, 136),
            Rgb::new(59, 82, 139),
            Rgb::new(54, 93, 141),
            Rgb::new(49, 104, 142),
            Rgb::new(44, 114, 142),
            Rgb::new(40, 124, 142),
            Rgb::new(36, 134, 142),
            Rgb::new(33, 144, 140),
            Rgb::new(31, 154, 138),
            Rgb::new(32, 164, 134),
            Rgb::new(39, 173, 129),
            Rgb::new(53, 183, 121),
            Rgb::new(71, 193, 110),
            Rgb::new(93, 200, 99),
            Rgb::new(117, 208, 84),
            Rgb::new(143, 215, 68),
            Rgb::new(170, 220, 50),
            Rgb::new(199, 224, 32),
            Rgb::new(227, 228, 24),
            Rgb::new(253, 231, 37),
        ];

        if t < 0.0 || t >= 1.0 {
            return Rgb::new(255, 0, 0);
        }
        let idx_float = t * (color_pallet.len() - 1) as f32;
        let idx = idx_float.floor() as usize;
        let s = idx_float.fract();
        Rgb::new(
            (color_pallet[idx].r as f32 * (1.0 - s) + color_pallet[idx + 1].r as f32 * s) as u8,
            (color_pallet[idx].g as f32 * (1.0 - s) + color_pallet[idx + 1].g as f32 * s) as u8,
            (color_pallet[idx].b as f32 * (1.0 - s) + color_pallet[idx + 1].b as f32 * s) as u8,
        )
    }
}
