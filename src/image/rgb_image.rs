use std::ops::{Deref, DerefMut};

use super::Image;
use crate::{THREAD_POOL, util::Rgb};

pub struct RgbImage {
    image: Image<glam::Vec3>,
}

impl RgbImage {
    pub fn new(width: usize, height: usize, pixels: Vec<glam::Vec3>) -> Self {
        Self {
            image: Image {
                width,
                height,
                pixels,
            },
        }
    }

    pub fn load_exr(filename: impl AsRef<std::path::Path>) -> exr::error::Result<Self> {
        let image = exr::prelude::read_first_rgba_layer_from_file(
            filename,
            |resolution, _channels| Image {
                width: resolution.width(),
                height: resolution.height(),
                pixels: vec![glam::Vec3::ZERO; resolution.width() * resolution.height()],
            },
            |image, position, (r, g, b, _a): (f32, f32, f32, f32)| {
                let index = position.y() * image.width + position.x();
                image.pixels[index] = glam::vec3(r, g, b);
            },
        )?;

        Ok(Self {
            image: image.layer_data.channel_data.pixels,
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

        THREAD_POOL.parallel_for_2d_coarse(
            self.image.width,
            self.image.height,
            &mut buffer,
            |x, y, pixel| {
                let rgb = Rgb::from(self.image.pixels[y * self.image.width + x]);
                *pixel = rgb.to_array();
            },
        );
        file.write_all(buffer.as_flattened())?;

        Ok(())
    }

    fn save_exr(&self, filename: impl AsRef<std::path::Path>) -> exr::error::Result<()> {
        use exr::image::write::WritableImage;

        let channels = exr::image::SpecificChannels::rgb(|position: exr::prelude::Vec2<usize>| {
            let rgb = self.image.pixels[position.y() * self.image.width + position.x()];
            (rgb.x, rgb.y, rgb.z)
        });

        let layer = exr::image::Layer::new(
            (self.image.width, self.image.height),
            exr::meta::header::LayerAttributes::default(),
            exr::image::Encoding::SMALL_LOSSLESS,
            channels,
        );

        let exr_image = exr::image::Image::from_layer(layer);

        exr_image.write().to_file(filename)
    }
}

impl Deref for RgbImage {
    type Target = Image<glam::Vec3>;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl DerefMut for RgbImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}
