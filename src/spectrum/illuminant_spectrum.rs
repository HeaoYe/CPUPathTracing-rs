use super::{CIE_1924_V, K_CD, Spectrum};

pub struct IlluminantSpectrum<'a> {
    illuminant: &'a Spectrum<'a>,
    lambda_min: f32,
    lambda_max: f32,
    k: f32,
    maximum: f32,
}

impl<'a> IlluminantSpectrum<'a> {
    pub(super) fn new(
        illuminant: &'a Spectrum,
        luminance: f32,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        let mut total = 0.0;
        let mut lambda = lambda_min;
        while lambda <= lambda_max {
            total += illuminant.eval(lambda) * CIE_1924_V.eval(lambda);
            lambda += 1.0;
        }

        let k = luminance / (total * K_CD);
        let maximum = k * illuminant.max();

        Self {
            illuminant,
            lambda_min,
            lambda_max,
            k,
            maximum,
        }
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        if lambda < self.lambda_min || lambda > self.lambda_max {
            0.0
        } else {
            self.k * self.illuminant.eval(lambda)
        }
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
