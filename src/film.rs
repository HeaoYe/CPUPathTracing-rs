pub struct Film {
    width: usize,
    height: usize,
    pixels: Vec<glam::Vec3>,
}

impl Film {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![glam::Vec3::ZERO; width * height],
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
            let data = (pixel * 255.0)
                .clamp(glam::Vec3::ZERO, glam::Vec3::splat(255.0))
                .as_u8vec3();
            writer.write_all(&data.to_array())?;
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

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<glam::Vec3> {
        let index = self.coord_to_index(x, y)?;
        Some(self.pixels[index])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: glam::Vec3) -> Option<()> {
        let index = self.coord_to_index(x, y)?;
        self.pixels[index] = color;
        Some(())
    }

    pub fn as_slice_mut(&mut self) -> &mut [glam::Vec3] {
        &mut self.pixels
    }

    fn coord_to_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(x + y * self.width)
        }
    }
}
