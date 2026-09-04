use super::{Image, RgbImage};
use crate::{
    THREAD_POOL,
    color::ColorLut,
    spectrum::{
        LAMBDA_MAX, LAMBDA_MIN, SigmoidPolynomialSpectrum, Spectrum, SpectrumSample,
        WavelengthSample, Y_CMF,
    },
};

#[derive(Clone, Copy, Default)]
struct Pixel {
    sigmoid: SigmoidPolynomialSpectrum,
    k: f32,
}

pub struct RgbIlluminantImage<'a> {
    image: Image<Pixel>,
    illuminant: &'a Spectrum<'a>,
}

impl<'a> RgbIlluminantImage<'a> {
    pub fn new(color_lut: &'a ColorLut<'a>, image: &RgbImage) -> Self {
        let width = image.width();
        let height = image.height();

        let illuminant = color_lut.illuminant();

        let mut illuminant_y = 0.0;

        for lambda in LAMBDA_MIN..=LAMBDA_MAX {
            illuminant_y += illuminant.eval(lambda as f32) * Y_CMF.eval(lambda as f32);
        }

        let illuminant_k = 1.0 / illuminant_y;

        let mut result = Image::new(width, height, vec![Pixel::default(); width * height]);

        THREAD_POOL.parallel_for_1d_coarse(&mut result.pixels, |index, pixel| {
            let mut linear_rgb = image.pixels[index];
            if image.color_space().name() != color_lut.color_space().name() {
                let xyz = image.color_space().xyz_from_rgb(linear_rgb);
                linear_rgb = color_lut.color_space().rgb_from_xyz(xyz);
                linear_rgb = linear_rgb.as_vec3().max(glam::Vec3::ZERO).into();
            }

            let scale = 2.0 * linear_rgb.r().max(linear_rgb.g()).max(linear_rgb.b());
            let lut_rgb = linear_rgb / scale;
            let Spectrum::Sigmoid(sigmoid) = color_lut.lookup_linear(lut_rgb) else {
                unreachable!()
            };

            *pixel = Pixel {
                sigmoid,
                k: scale * illuminant_k,
            };
        });

        Self {
            image: result,
            illuminant,
        }
    }

    pub fn sample(&self, x: usize, y: usize, wavelength: &WavelengthSample) -> SpectrumSample {
        let x = x.min(self.image.width - 1);
        let y = y.min(self.image.height - 1);

        let pixel = self.image.pixels[y * self.image.width + x];

        Spectrum::sample(&Spectrum::Sigmoid(pixel.sigmoid), wavelength)
            * self.illuminant.sample(wavelength)
            * pixel.k
    }

    pub fn sample_image_point(
        &self,
        image_point: glam::Vec2,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        self.sample(image_point.x as usize, image_point.y as usize, wavelength)
    }

    pub fn width(&self) -> usize {
        self.image.width
    }

    pub fn height(&self) -> usize {
        self.image.height
    }
}
