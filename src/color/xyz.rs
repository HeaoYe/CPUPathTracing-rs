use super::Chromaticity;
use crate::spectrum::{
    Spectrum, SpectrumSample, WAVELENGTH_SAMPLE_COUNT, WavelengthSample, X_CMF, Y_CMF, Z_CMF,
};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Default, Clone, Copy)]
pub struct Xyz {
    data: glam::Vec3,
}

impl Xyz {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            data: glam::vec3(x, y, z),
        }
    }

    pub fn from_vec3(data: glam::Vec3) -> Self {
        Self { data }
    }

    pub fn from_chromaticity(chromaticity: Chromaticity, luminance: f32) -> Self {
        let x = chromaticity.x();
        let y = chromaticity.y();
        let z = 1.0 - x - y;

        let scale = luminance / y;

        Self::new(x * scale, luminance, z * scale)
    }

    pub fn from_spectrum(spectrum: &Spectrum<'_>) -> Self {
        let mut result = Self::default();

        for lambda in 360..=830 {
            let lambda = lambda as f32;
            let value = spectrum.eval(lambda);

            result.data.x += value * X_CMF.eval(lambda);
            result.data.y += value * Y_CMF.eval(lambda);
            result.data.z += value * Z_CMF.eval(lambda);
        }

        result
    }

    pub fn from_spectrum_sample(
        spectrum_sample: SpectrumSample,
        wavelength: &WavelengthSample,
    ) -> Self {
        let mut xyz = glam::Vec3::ZERO;

        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            let pdf = wavelength.pdf(i);
            if pdf == 0.0 {
                continue;
            }

            let value = spectrum_sample[i] / pdf;
            let lambda = wavelength.lambda(i);

            xyz.x += value * X_CMF.eval(lambda);
            xyz.y += value * Y_CMF.eval(lambda);
            xyz.z += value * Z_CMF.eval(lambda);
        }

        xyz /= WAVELENGTH_SAMPLE_COUNT as f32;

        Self::from_vec3(xyz)
    }

    pub fn x(&self) -> f32 {
        self.data.x
    }

    pub fn y(&self) -> f32 {
        self.data.y
    }

    pub fn z(&self) -> f32 {
        self.data.z
    }

    pub fn as_vec3(&self) -> glam::Vec3 {
        self.data
    }

    pub fn as_dvec3(&self) -> glam::DVec3 {
        self.data.as_dvec3()
    }

    pub fn is_nan(&self) -> bool {
        self.data.is_nan()
    }
}

impl From<glam::Vec3> for Xyz {
    fn from(value: glam::Vec3) -> Self {
        Self::from_vec3(value)
    }
}

impl From<Xyz> for glam::Vec3 {
    fn from(value: Xyz) -> Self {
        value.data
    }
}

impl From<Chromaticity> for Xyz {
    fn from(value: Chromaticity) -> Self {
        Self::from_chromaticity(value, 1.0)
    }
}

impl Add for Xyz {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_vec3(self.data + rhs.data)
    }
}

impl Sub for Xyz {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_vec3(self.data - rhs.data)
    }
}

impl Mul<f32> for Xyz {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_vec3(self.data * rhs)
    }
}

impl Div<f32> for Xyz {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from_vec3(self.data / rhs)
    }
}

impl AddAssign for Xyz {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl SubAssign for Xyz {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl MulAssign<f32> for Xyz {
    fn mul_assign(&mut self, rhs: f32) {
        self.data *= rhs;
    }
}

impl DivAssign<f32> for Xyz {
    fn div_assign(&mut self, rhs: f32) {
        self.data /= rhs;
    }
}
