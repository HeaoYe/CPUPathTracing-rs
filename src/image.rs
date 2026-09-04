mod rgb_illuminant_image;
mod rgb_image;

pub use rgb_illuminant_image::RgbIlluminantImage;
pub use rgb_image::RgbImage;

pub struct Image<T> {
    width: usize,
    height: usize,
    pixels: Vec<T>,
}

impl<T> Image<T> {
    pub fn new(width: usize, height: usize, pixels: Vec<T>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn resolution(&self) -> glam::USizeVec2 {
        glam::usizevec2(self.width, self.height)
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<&T> {
        self.pixels.get(y * self.width + x)
    }

    pub fn get_pixel_wrapped(&self, x: i32, y: i32) -> &T {
        let x = x.clamp(0, self.width as i32 - 1) as usize;
        let y = y.clamp(0, self.height as i32 - 1) as usize;
        &self.pixels[y * self.width + x]
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.pixels
    }
}
