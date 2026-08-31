use crate::util::RGB;

#[derive(Default, Clone)]
pub struct Pixel {
    color_sum: glam::Vec3,
    sample_count: usize,
}

pub enum PixelSample {
    Radiance(glam::Vec3),
    RGB(crate::util::RGB),
}

impl Pixel {
    pub fn add_sample(&mut self, sample: PixelSample) {
        match sample {
            PixelSample::Radiance(radiance) => self.color_sum += radiance,
            PixelSample::RGB(rgb) => self.color_sum += glam::Vec3::from(rgb),
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
        use std::{
            fs::File,
            io::{BufWriter, Write},
        };

        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(format!("P6\n{} {}\n255\n", self.width, self.height).as_bytes())?;
        for pixel in &self.pixels {
            let rgb = RGB::from(pixel.average());
            writer.write_all(&rgb.to_array())?;
        }
        writer.flush()?;

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
