use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

pub struct SigmoidPolynomialSpectrum {
    c0: f32,
    c1: f32,
    c2: f32,
}

impl SigmoidPolynomialSpectrum {
    pub fn new(c0: f32, c1: f32, c2: f32) -> Self {
        Self { c0, c1, c2 }
    }

    pub fn eval(&self, lambda: f32) -> f32 {
        let u = (lambda - LAMBDA_MIN as f32) / (LAMBDA_MAX - LAMBDA_MIN) as f32;
        Self::sigmoid(((self.c2 * u) + self.c1) * u + self.c0)
    }

    pub fn max(&self) -> f32 {
        let left = self.eval(LAMBDA_MIN as f32);
        let right = self.eval(LAMBDA_MAX as f32);

        if self.c2 >= 0.0 {
            return left.max(right);
        }

        let mid = -self.c1 / (2.0 * self.c2);
        if mid <= LAMBDA_MIN as f32 {
            left
        } else if mid >= LAMBDA_MAX as f32 {
            right
        } else {
            self.eval(mid)
        }
    }

    fn sigmoid(x: f32) -> f32 {
        if x.is_infinite() {
            if x > 0.0 { 1.0 } else { 0.0 }
        } else {
            0.5 * x / (1.0 + x * x).sqrt() + 0.5
        }
    }
}
