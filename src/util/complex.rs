use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Default, Clone, Copy)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::Output {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::Output {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::Output {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Mul<f32> for Complex {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::Output {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let inv_denom = 1.0 / (rhs.re * rhs.re + rhs.im * rhs.im);
        Self::Output {
            re: (self.re * rhs.re + self.im * rhs.im) * inv_denom,
            im: (self.im * rhs.re - self.re * rhs.im) * inv_denom,
        }
    }
}

impl Div<f32> for Complex {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::Output {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl Complex {
    pub fn conjugate(self) -> Self {
        Self::new(self.re, -self.im)
    }

    pub fn norm_squared(self) -> f32 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(self) -> f32 {
        self.norm_squared().sqrt()
    }

    pub fn sqrt(self) -> Self {
        let norm = self.norm();
        Self {
            re: ((norm + self.re) * 0.5).sqrt(),
            im: ((norm - self.re) * 0.5).sqrt().copysign(self.im),
        }
    }
}

impl From<f32> for Complex {
    fn from(value: f32) -> Self {
        Self { re: value, im: 0.0 }
    }
}
