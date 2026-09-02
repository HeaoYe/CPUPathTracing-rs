use crate::{THREAD_POOL, util::Rgb};
use std::io::Write;

#[derive(Default, Clone)]
pub struct Pixel {
    color_sum: glam::Vec3,
    sample_count: usize,
}

pub enum PixelSample {
    Radiance(glam::Vec3),
    Rgb(crate::util::Rgb),
}

impl Pixel {
    pub fn add_sample(&mut self, sample: PixelSample) {
        match sample {
            PixelSample::Radiance(radiance) => {
                if radiance.is_nan() {
                    return;
                }
                self.color_sum += radiance
            }
            PixelSample::Rgb(rgb) => self.color_sum += glam::Vec3::from(rgb),
        }
        self.sample_count += 1;
    }

    pub fn average(&self) -> glam::Vec3 {
        if self.sample_count == 0 {
            glam::Vec3::ZERO
        } else {
            self.color_sum / self.sample_count as f32
        }
    }
}

pub struct Film {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
}

impl Film {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Default::default(); width * height],
        }
    }

    pub fn save(&self, filename: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let mut file = std::fs::File::create(filename)?;
        let mut buffer = vec![[0u8; 3]; self.width * self.height];
        file.write_all(format!("P6\n{} {}\n255\n", self.width, self.height).as_bytes())?;

        THREAD_POOL.parallel_for_2d_coarse(self.width, self.height, &mut buffer, |x, y, pixel| {
            let rgb = Rgb::from(self.pixels[y * self.width + x].average());
            *pixel = rgb.to_array();
        });
        file.write_all(buffer.as_flattened())?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.pixels.clear();
        self.pixels
            .resize(self.width * self.height, Default::default());
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn as_slice_mut(&mut self) -> &mut [Pixel] {
        &mut self.pixels
    }
}
