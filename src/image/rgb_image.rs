use super::Image;
use crate::{
    THREAD_POOL,
    color::{Chromaticity, ColorSpace, DCI_P3, LinearRgb, SRGB, TransferFunction},
};
use std::ops::{Deref, DerefMut};

pub struct RgbImage<'a> {
    image: Image<LinearRgb>,
    color_space: &'a ColorSpace,
}

impl<'a> RgbImage<'a> {
    pub fn new(
        width: usize,
        height: usize,
        pixels: Vec<LinearRgb>,
        color_space: &'a ColorSpace,
    ) -> Self {
        Self {
            image: Image {
                width,
                height,
                pixels,
            },
            color_space,
        }
    }

    pub fn load_exr(filename: impl AsRef<std::path::Path>) -> exr::error::Result<Self> {
        let image = exr::prelude::read_first_rgba_layer_from_file(
            filename,
            |resolution, _channels| Image {
                width: resolution.width(),
                height: resolution.height(),
                pixels: vec![LinearRgb::default(); resolution.width() * resolution.height()],
            },
            |image, position, (r, g, b, _a): (f32, f32, f32, f32)| {
                let index = position.y() * image.width + position.x();
                image.pixels[index] = LinearRgb::new(r, g, b);
            },
        )?;

        let mut rgb_image = image.layer_data.channel_data.pixels;
        let color_space = if let Some(chroma) = image.attributes.chromaticities {
            let image_color_space = ColorSpace::new(
                "",
                Chromaticity::new(chroma.red.x(), chroma.red.y()),
                Chromaticity::new(chroma.green.x(), chroma.green.y()),
                Chromaticity::new(chroma.blue.x(), chroma.blue.y()),
                Chromaticity::new(chroma.white.x(), chroma.white.y()),
                TransferFunction::default(),
            );
            THREAD_POOL.parallel_for_1d_coarse(&mut rgb_image.pixels, |_, linear_rgb| {
                let xyz = image_color_space.xyz_from_rgb(*linear_rgb);
                *linear_rgb = DCI_P3.rgb_from_xyz(xyz);
            });
            println!(
                "Load from chromaticties: red: {:?}, green: {:?}, blue: {:?}, white: {:?}, convert to DCI-P3",
                image_color_space.red(),
                image_color_space.green(),
                image_color_space.blue(),
                image_color_space.white()
            );
            &*DCI_P3
        } else {
            &*SRGB
        };

        Ok(Self {
            image: rgb_image,
            color_space,
        })
    }

    pub fn save(&self, filename: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let Some(extension) = filename.as_ref().extension() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidFilename,
                "please specify a file extension",
            ));
        };

        match extension.to_str() {
            Some("ppm") => self.save_ppm(filename),
            Some("exr") => self.save_exr(filename).map_err(std::io::Error::other),
            Some(ext) => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("unsupported image format: .{ext}"),
            )),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "file extension is not valid UTF-8",
            )),
        }
    }

    fn save_ppm(&self, filename: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(filename)?;
        let mut buffer = vec![[0u8; 3]; self.image.width * self.image.height];
        file.write_all(
            format!("P6\n{} {}\n255\n", self.image.width, self.image.height).as_bytes(),
        )?;

        THREAD_POOL.parallel_for_1d_coarse(&mut buffer, |index, pixel| {
            let mut linear_rgb = self.image.pixels[index];
            if self.color_space.name() != "sRGB" {
                let xyz = self.color_space.xyz_from_rgb(linear_rgb);
                linear_rgb = SRGB.rgb_from_xyz(xyz);
            }
            *pixel = SRGB
                .encode(linear_rgb)
                .to_quantized(8)
                .as_u8vec3()
                .to_array();
        });
        file.write_all(buffer.as_flattened())?;

        Ok(())
    }

    fn save_exr(&self, filename: impl AsRef<std::path::Path>) -> exr::error::Result<()> {
        use exr::image::write::WritableImage;

        let channels = exr::image::SpecificChannels::rgb(|position: exr::prelude::Vec2<usize>| {
            let rgb = self.image.pixels[position.y() * self.image.width + position.x()];
            (rgb.r(), rgb.g(), rgb.b())
        });

        let layer = exr::image::Layer::new(
            (self.image.width, self.image.height),
            exr::meta::header::LayerAttributes::default(),
            exr::image::Encoding::SMALL_LOSSLESS,
            channels,
        );

        let mut exr_image = exr::image::Image::from_layer(layer);

        exr_image.attributes.chromaticities = Some(exr::meta::attribute::Chromaticities {
            red: exr::math::Vec2(self.color_space.red().x(), self.color_space.red().y()),
            green: exr::math::Vec2(self.color_space.green().x(), self.color_space.green().y()),
            blue: exr::math::Vec2(self.color_space.blue().x(), self.color_space.blue().y()),
            white: exr::math::Vec2(self.color_space.white().x(), self.color_space.white().y()),
        });

        exr_image.write().to_file(filename)
    }
}

impl Deref for RgbImage<'_> {
    type Target = Image<LinearRgb>;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl DerefMut for RgbImage<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}
