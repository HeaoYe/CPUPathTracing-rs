use super::Spectrum;

pub struct DenselySampledSpectrum {
    lambda_min: u32,
    maximum: f32,
    values: Vec<f32>,
}

impl DenselySampledSpectrum {
    pub(super) fn from_values(values: Vec<f32>, lambda_min: u32, lambda_max: u32) -> Self {
        assert!(values.len() as u32 == lambda_max - lambda_min + 1);
        let maximum = *values.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

        Self {
            lambda_min,
            maximum,
            values,
        }
    }

    pub(super) fn from_spectrum(spectrum: &Spectrum, lambda_min: u32, lambda_max: u32) -> Self {
        let mut values = vec![0.0; (lambda_max - lambda_min) as usize + 1];
        let mut maximum = spectrum.eval(lambda_min as f32);
        for (i, value) in values.iter_mut().enumerate() {
            *value = spectrum.eval(lambda_min as f32 + i as f32);
            if maximum < *value {
                maximum = *value;
            }
        }
        Self {
            lambda_min,
            maximum,
            values,
        }
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        let offset = (lambda - self.lambda_min as f32).round();
        if offset < 0.0 {
            0.0
        } else {
            self.values.get(offset as usize).copied().unwrap_or(0.0)
        }
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
