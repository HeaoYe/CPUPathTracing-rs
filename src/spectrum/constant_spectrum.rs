pub struct ConstantSpectrum {
    constant: f32,
}

impl ConstantSpectrum {
    pub(super) fn new(constant: f32) -> Self {
        Self { constant }
    }

    pub(super) fn eval(&self) -> f32 {
        self.constant
    }

    pub(super) fn max(&self) -> f32 {
        self.constant
    }
}
