use crate::{
    THREAD_POOL,
    color::{ColorSpace, EncodedRgb, LinearRgb, SRGB, Xyz},
    image::RgbImage,
};

#[derive(Default, Clone)]
pub struct Pixel {
    xyz_sum: glam::DVec3,
    sample_count: usize,
}

pub enum PixelSample<'a> {
    Radiance(glam::Vec3),
    Rgb(EncodedRgb, &'a ColorSpace),
}

impl Pixel {
    pub fn add_sample(&mut self, sample: PixelSample) {
        match sample {
            PixelSample::Radiance(radiance) => {
                if radiance.is_nan() {
                    return;
                }
                self.xyz_sum += SRGB.xyz_from_rgb(radiance.into()).as_dvec3();
            }
            PixelSample::Rgb(encoded_rgb, color_space) => {
                self.xyz_sum += color_space
                    .xyz_from_rgb(color_space.decode(encoded_rgb))
                    .as_dvec3()
            }
        }
        self.sample_count += 1;
    }

    pub fn average(&self) -> glam::Vec3 {
        if self.sample_count == 0 {
            glam::Vec3::ZERO
        } else {
            (self.xyz_sum / self.sample_count as f64).as_vec3()
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

    pub fn write_rgb_buffer(
        &self,
        width: usize,
        height: usize,
        dst: &mut [u32],
        color_space: &ColorSpace,
    ) {
        debug_assert_eq!(dst.len(), width * height);
        THREAD_POOL.parallel_for_2d_coarse(width, height, dst, move |x, y, buffer| {
            let x = x * self.width / width;
            let y = y * self.height / height;
            let index = y * self.width + x;
            let xyz = Xyz::from(self.pixels[index].average());
            let [r, g, b] = color_space
                .encode(color_space.rgb_from_xyz(xyz))
                .to_quantized(8)
                .to_array();
            *buffer = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        });
    }

    pub fn save(
        &self,
        filename: impl AsRef<std::path::Path>,
        color_space: &ColorSpace,
    ) -> std::io::Result<()> {
        let mut buffer = vec![LinearRgb::default(); self.width * self.height];
        THREAD_POOL.parallel_for_1d_coarse(&mut buffer, |index, pixel| {
            let xyz = Xyz::from(self.pixels[index].average());
            *pixel = color_space.rgb_from_xyz(xyz);
        });
        let image = RgbImage::new(self.width, self.height, buffer, color_space);
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
