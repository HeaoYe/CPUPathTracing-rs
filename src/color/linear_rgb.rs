use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Default, Clone, Copy)]
pub struct LinearRgb {
    data: glam::Vec3,
}

impl LinearRgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            data: glam::vec3(r, g, b),
        }
    }

    pub fn from_vec3(data: glam::Vec3) -> Self {
        Self { data }
    }

    pub fn r(&self) -> f32 {
        self.data.x
    }

    pub fn g(&self) -> f32 {
        self.data.y
    }

    pub fn b(&self) -> f32 {
        self.data.z
    }

    pub fn as_vec3(&self) -> glam::Vec3 {
        self.data
    }

    pub fn as_dvec3(&self) -> glam::DVec3 {
        self.data.as_dvec3()
    }
}

impl From<glam::Vec3> for LinearRgb {
    fn from(value: glam::Vec3) -> Self {
        Self::from_vec3(value)
    }
}

impl From<LinearRgb> for glam::Vec3 {
    fn from(value: LinearRgb) -> Self {
        value.data
    }
}

impl Add for LinearRgb {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_vec3(self.data + rhs.data)
    }
}

impl Sub for LinearRgb {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_vec3(self.data - rhs.data)
    }
}

impl Mul<f32> for LinearRgb {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_vec3(self.data * rhs)
    }
}

impl Div<f32> for LinearRgb {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from_vec3(self.data / rhs)
    }
}

impl AddAssign for LinearRgb {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign for LinearRgb {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<f32> for LinearRgb {
    fn mul_assign(&mut self, rhs: f32) {
        self.data *= rhs;
    }
}

impl DivAssign<f32> for LinearRgb {
    fn div_assign(&mut self, rhs: f32) {
        self.data /= rhs;
    }
}
