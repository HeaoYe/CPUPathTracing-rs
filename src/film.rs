use super::rgb::RGB;

#[derive(Default, Clone)]
pub struct Pixel {
    color_sum: glam::Vec3,
    sample_count: usize,
}

impl Pixel {
    pub fn add_sample(&mut self, color: glam::Vec3) {
        self.color_sum += color;
        self.sample_count += 1;
    }

    pub fn average(&self) -> glam::Vec3 {
        self.color_sum / self.sample_count as f32
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
            pixels: vec![Pixel::default(); width * height],
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
