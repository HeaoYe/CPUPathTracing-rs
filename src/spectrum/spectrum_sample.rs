use super::WAVELENGTH_SAMPLE_COUNT;
use std::{
    array,
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectrumSample {
    data: [f32; WAVELENGTH_SAMPLE_COUNT],
}

impl SpectrumSample {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);

    pub const fn splat(value: f32) -> Self {
        Self {
            data: [value; WAVELENGTH_SAMPLE_COUNT],
        }
    }

    pub const fn from_array(data: [f32; WAVELENGTH_SAMPLE_COUNT]) -> Self {
        Self { data }
    }

    pub fn max_element(self) -> f32 {
        self.data.into_iter().reduce(f32::max).unwrap()
    }

    pub fn ln(self) -> Self {
        Self {
            data: self.data.map(f32::ln),
        }
    }

    pub fn exp(self) -> Self {
        Self {
            data: self.data.map(f32::exp),
        }
    }

    pub const fn as_array(&self) -> &[f32; WAVELENGTH_SAMPLE_COUNT] {
        &self.data
    }
}

impl Default for SpectrumSample {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Index<usize> for SpectrumSample {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for SpectrumSample {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Add for SpectrumSample {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_array(array::from_fn(|i| self[i] + rhs[i]))
    }
}

impl Sub for SpectrumSample {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_array(array::from_fn(|i| self[i] - rhs[i]))
    }
}

impl Mul for SpectrumSample {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_array(array::from_fn(|i| self[i] * rhs[i]))
    }
}

impl Div for SpectrumSample {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from_array(array::from_fn(|i| self[i] / rhs[i]))
    }
}

impl Mul<f32> for SpectrumSample {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_array(self.data.map(|value| value * rhs))
    }
}

impl Div<f32> for SpectrumSample {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from_array(self.data.map(|value| value / rhs))
    }
}

impl AddAssign for SpectrumSample {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            self[i] += rhs[i];
        }
    }
}

impl SubAssign for SpectrumSample {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            self[i] -= rhs[i];
        }
    }
}

impl MulAssign for SpectrumSample {
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            self[i] *= rhs[i];
        }
    }
}

impl DivAssign for SpectrumSample {
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            self[i] /= rhs[i];
        }
    }
}

impl MulAssign<f32> for SpectrumSample {
    fn mul_assign(&mut self, rhs: f32) {
        for value in &mut self.data {
            *value *= rhs;
        }
    }
}

impl DivAssign<f32> for SpectrumSample {
    fn div_assign(&mut self, rhs: f32) {
        for value in &mut self.data {
            *value /= rhs;
        }
    }
}

impl Sum for SpectrumSample {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Self::ZERO;
        for value in iter {
            sum += value;
        }
        sum
    }
}
