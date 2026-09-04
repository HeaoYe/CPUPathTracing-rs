mod analytic_spectrum;
mod blackbody_spectrum;
mod cmfs;
mod constant_spectrum;
mod densely_sampled_spectrum;
mod illuminant;
mod illuminant_spectrum;
mod piecewise_linear_spectrum;
mod wavelength;

pub use analytic_spectrum::AnalyticSpectrum;
pub use blackbody_spectrum::BlackbodySpectrum;
pub use cmfs::{X_CMF, Y_CMF, Z_CMF};
pub use constant_spectrum::ConstantSpectrum;
pub use densely_sampled_spectrum::DenselySampledSpectrum;
pub use illuminant::{
    CIE_1924_V, CIE_STD_ILLUMNT_A, CIE_STD_ILLUMNT_D50, CIE_STD_ILLUMNT_D65, K_CD,
};
pub use illuminant_spectrum::IlluminantSpectrum;
pub use piecewise_linear_spectrum::{PiecewiseLinearSpectrum, SamplePoint};
pub use wavelength::{LAMBDA_MAX, LAMBDA_MIN};

pub enum Spectrum<'a> {
    Constant(ConstantSpectrum),
    Dense(DenselySampledSpectrum),
    PiecewiseLinear(PiecewiseLinearSpectrum),
    Blackbody(BlackbodySpectrum),
    Analytic(AnalyticSpectrum),
    Illuminant(IlluminantSpectrum<'a>),
}

impl Spectrum<'_> {
    pub fn eval(&self, lambda: f32) -> f32 {
        if lambda < LAMBDA_MIN as f32 || lambda > LAMBDA_MAX as f32 {
            0.0
        } else {
            match self {
                Self::Constant(spectrum) => spectrum.eval(),
                Self::Dense(spectrum) => spectrum.eval(lambda),
                Self::PiecewiseLinear(spectrum) => spectrum.eval(lambda),
                Self::Blackbody(spectrum) => spectrum.eval(lambda),
                Self::Analytic(spectrum) => spectrum.eval(lambda),
                Self::Illuminant(spectrum) => spectrum.eval(lambda),
            }
        }
    }

    pub fn max(&self) -> f32 {
        match self {
            Self::Constant(spectrum) => spectrum.max(),
            Self::Dense(spectrum) => spectrum.max(),
            Self::PiecewiseLinear(spectrum) => spectrum.max(),
            Self::Blackbody(spectrum) => spectrum.max(),
            Self::Analytic(spectrum) => spectrum.max(),
            Self::Illuminant(spectrum) => spectrum.max(),
        }
    }
}

impl<'a> Spectrum<'a> {
    pub fn constant(constant: f32) -> Self {
        Self::Constant(ConstantSpectrum::new(constant))
    }

    pub fn dense_from_values(
        values: impl Into<Vec<f32>>,
        lambda_min: u32,
        lambda_max: u32,
    ) -> Self {
        Self::Dense(DenselySampledSpectrum::from_values(
            values.into(),
            lambda_min,
            lambda_max,
        ))
    }

    pub fn dense_from_spectrum(spectrum: &Spectrum, lambda_min: u32, lambda_max: u32) -> Self {
        Self::Dense(DenselySampledSpectrum::from_spectrum(
            spectrum, lambda_min, lambda_max,
        ))
    }

    pub fn piecewise_linear_from_samples(samples: impl Into<Vec<SamplePoint>>) -> Self {
        Self::PiecewiseLinear(PiecewiseLinearSpectrum::from_samples(samples.into()))
    }

    pub fn piecewise_linear_from_values(
        lambdas: impl Into<Vec<f32>>,
        values: impl Into<Vec<f32>>,
    ) -> Self {
        Self::PiecewiseLinear(PiecewiseLinearSpectrum::from_values(
            lambdas.into(),
            values.into(),
        ))
    }

    pub fn blackbody(temperature: f32) -> Self {
        Self::Blackbody(BlackbodySpectrum::new(temperature))
    }

    pub fn analytic(
        expression: impl Fn(f32) -> f32 + Send + Sync + 'static,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        Self::Analytic(AnalyticSpectrum::new(expression, lambda_min, lambda_max))
    }

    pub fn analytic_with_maximum(
        expression: impl Fn(f32) -> f32 + Send + Sync + 'static,
        maximum: f32,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        Self::Analytic(AnalyticSpectrum::with_maximum(
            expression, maximum, lambda_min, lambda_max,
        ))
    }

    pub fn illuminant(illuminant: &'a Spectrum, luminance: f32) -> Self {
        Self::illuminant_with_range(illuminant, luminance, LAMBDA_MIN as f32, LAMBDA_MAX as f32)
    }

    pub fn illuminant_with_range(
        illuminant: &'a Spectrum,
        luminance: f32,
        lambda_min: f32,
        lambda_max: f32,
    ) -> Self {
        Self::Illuminant(IlluminantSpectrum::new(
            illuminant, luminance, lambda_min, lambda_max,
        ))
    }
}
