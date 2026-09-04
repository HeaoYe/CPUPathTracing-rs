pub struct SamplePoint {
    pub lambda: f32,
    pub value: f32,
}

pub struct PiecewiseLinearSpectrum {
    lambda_min: f32,
    lambda_max: f32,
    maximum: f32,
    lambdas: Vec<f32>,
    values: Vec<f32>,
}

impl PiecewiseLinearSpectrum {
    pub(super) fn from_samples(samples: Vec<SamplePoint>) -> Self {
        let mut lambdas = Vec::with_capacity(samples.len());
        let mut values = Vec::with_capacity(samples.len());

        for sample in samples {
            lambdas.push(sample.lambda);
            values.push(sample.value);
        }

        Self::from_values(lambdas, values)
    }

    pub(super) fn from_values(lambdas: Vec<f32>, values: Vec<f32>) -> Self {
        let lambda_min = lambdas[0];
        let lambda_max = *lambdas.last().unwrap();
        let maximum = values.iter().copied().max_by(f32::total_cmp).unwrap();

        Self {
            lambda_min,
            lambda_max,
            maximum,
            lambdas,
            values,
        }
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        if lambda < self.lambda_min || lambda > self.lambda_max {
            return 0.0;
        }

        if lambda == self.lambda_max {
            return *self.values.last().unwrap();
        }

        let right = self
            .lambdas
            .partition_point(|&sample_lambda| sample_lambda <= lambda);
        let left = right - 1;
        let t = (lambda - self.lambdas[left]) / (self.lambdas[right] - self.lambdas[left]);
        self.values[left] * (1.0 - t) + self.values[right] * t
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
