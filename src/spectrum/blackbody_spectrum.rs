use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

pub struct BlackbodySpectrum {
    temperature: f32,
    maximum: f32,
}

impl BlackbodySpectrum {
    pub(super) fn new(temperature: f32) -> Self {
        let mut spectrum = Self {
            temperature,
            maximum: 0.0,
        };

        let mut maximum = spectrum
            .eval(LAMBDA_MIN as f32)
            .max(spectrum.eval(LAMBDA_MAX as f32));
        let mut lambda = LAMBDA_MIN as f32 + 0.1;
        while lambda < LAMBDA_MAX as f32 {
            let value = spectrum.eval(lambda);
            if maximum < value {
                maximum = value;
            }
            lambda += 0.1;
        }
        spectrum.maximum = maximum;

        spectrum
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        const C: f32 = 299792458.0;
        const H: f64 = 6.62607015e-34;
        const KB: f32 = 1.3806488e-23;

        let l = lambda * 1e-9;
        2.0 * H as f32 * C * C / (l.powi(5) * (H as f32 * C / (l * KB * self.temperature)).exp_m1())
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
