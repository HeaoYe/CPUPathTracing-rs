use super::{LAMBDA_MAX, LAMBDA_MIN, SigmoidPolynomialSpectrum, Spectrum, Y_CMF};
use crate::color::{ColorLut, LinearRgb};

pub struct RgbIlluminantSpectrum<'a> {
    sigmoid: SigmoidPolynomialSpectrum,
    illuminant: &'a Spectrum<'a>,
    k: f32,
    maximum: f32,
}

impl<'a> RgbIlluminantSpectrum<'a> {
    pub(super) fn new(color_lut: &'a ColorLut<'a>, linear_rgb: LinearRgb) -> Self {
        let illuminant = color_lut.illuminant();
        let scale = 2.0 * linear_rgb.max_element();
        let lut_rgb = linear_rgb / scale;
        let Spectrum::Sigmoid(sigmoid) = color_lut.lookup_linear(lut_rgb) else {
            unreachable!()
        };

        let mut k = 0.0;
        let mut maximum = 0.0_f32;

        for lambda in LAMBDA_MIN..=LAMBDA_MAX {
            let illumnt = illuminant.eval(lambda as f32);
            maximum = maximum.max(sigmoid.eval(lambda as f32) * illumnt);
            k += illumnt * Y_CMF.eval(lambda as f32);
        }

        k = scale / k;
        maximum *= k;

        Self {
            sigmoid,
            illuminant,
            k,
            maximum,
        }
    }

    pub(super) fn eval(&self, lambda: f32) -> f32 {
        self.sigmoid.eval(lambda) * self.illuminant.eval(lambda) * self.k
    }

    pub(super) fn max(&self) -> f32 {
        self.maximum
    }
}
