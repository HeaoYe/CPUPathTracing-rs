use super::Xyz;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Chromaticity {
    x: f32,
    y: f32,
}

impl Chromaticity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }
}

impl From<Xyz> for Chromaticity {
    fn from(xyz: Xyz) -> Self {
        let sum = xyz.x() + xyz.y() + xyz.z();

        Self {
            x: xyz.x() / sum,
            y: xyz.y() / sum,
        }
    }
}
