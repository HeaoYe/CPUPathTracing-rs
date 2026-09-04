use crate::{THREAD_POOL, image::RgbImage, util::Rgb};

#[derive(Default, Clone)]
pub struct Pixel {
    color_sum: glam::DVec3,
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
                self.color_sum += radiance.as_dvec3()
            }
            PixelSample::Rgb(rgb) => self.color_sum += glam::Vec3::from(rgb).as_dvec3(),
        }
        self.sample_count += 1;
    }

    pub fn average(&self) -> glam::Vec3 {
        if self.sample_count == 0 {
            glam::Vec3::ZERO
        } else {
            (self.color_sum / self.sample_count as f64).as_vec3()
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

    pub fn write_rgb_buffer(&self, width: usize, height: usize, dst: &mut [u32]) {
        debug_assert_eq!(dst.len(), width * height);
        THREAD_POOL.parallel_for_2d_coarse(width, height, dst, move |x, y, buffer| {
            let x = x * self.width / width;
            let y = y * self.height / height;
            let index = y * self.width + x;
            let pixel = self.pixels[index].average();
            let [r, g, b] = Rgb::from(pixel).to_array();
            *buffer = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        });
    }

    pub fn save(&self, filename: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let mut buffer = vec![glam::Vec3::ZERO; self.width * self.height];
        THREAD_POOL.parallel_for_2d_coarse(self.width, self.height, &mut buffer, |x, y, pixel| {
            *pixel = self.pixels[y * self.width + x].average();
        });
        let image = RgbImage::new(self.width, self.height, buffer);
        image.save(filename)
    }

    pub fn clear(&mut self) {
        self.pixels.clear();
        self.pixels
            .resize(self.width * self.height, Default::default());
    }

    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.clear();
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
