type Expression = Box<dyn Fn(f32) -> f32 + Send + Sync>;

pub struct AnalyticSpectrum {
    lambda_min: f32,
    lambda_max: f32,
    maximum: f32,
    expression: Expression,
}

impl AnalyticSpectrum {
    pub(super) fn new(
        expression: impl Fn(f32) -> f32 + Send + Sync + 'static,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        let expression: Expression = Box::new(expression);

        let mut maximum = expression(lambda_min).max(expression(lambda_max));
        let mut lambda = lambda_min + 0.1;
        while lambda < lambda_max {
            let value = expression(lambda);
            if maximum < value {
                maximum = value;
            }
            lambda += 0.1;
        }

        Self {
            lambda_min,
            lambda_max,
            maximum,
            expression,
        }
    }

    pub(super) fn with_maximum(
        expression: impl Fn(f32) -> f32 + Send + Sync + 'static,
        maximum: f32,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        assert!(lambda_min <= lambda_max);

        Self {
            lambda_min,
            lambda_max,
            maximum,
            expression: Box::new(expression),
        }
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        if lambda < self.lambda_min || lambda > self.lambda_max {
            0.0
        } else {
            (self.expression)(lambda)
        }
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
