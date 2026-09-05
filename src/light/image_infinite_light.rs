use super::LightSample;
use crate::{
    color::LUT_SRGB,
    image::{RgbIlluminantImage, RgbImage},
    light_sampler::MisCompensation,
    sample::AliasTable,
    spectrum::{SpectrumSample, WavelengthSample},
};

pub struct ImageInfiniteLight<'a> {
    image: RgbIlluminantImage<'a>,
    start_phi: f32,
    power: f32,
    gird_count: glam::USizeVec2,
    skip_compensation: bool,
    alias_table: AliasTable,
    alias_table_compensated: AliasTable,
}

const GIRD_SIZE: usize = 50;

impl<'a> ImageInfiniteLight<'a> {
    pub(crate) fn new(image: &'a RgbImage, start_phi: f32) -> Self {
        let gird_count = image.resolution() / GIRD_SIZE + 1;
        let mut light = Self {
            image: RgbIlluminantImage::new(&LUT_SRGB, image),
            start_phi,
            power: 0.0,
            gird_count,
            skip_compensation: true,
            alias_table: Default::default(),
            alias_table_compensated: Default::default(),
        };

        let mut girds_power = vec![0.0; gird_count.x * gird_count.y];
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel_power = image.get_pixel(x, y).unwrap().as_vec3().max_element();
                light.power += pixel_power;
                let gird_index = light.gird_idx_from_image_point(glam::vec2(x as f32, y as f32));
                girds_power[gird_index.y * gird_count.x + gird_index.x] += pixel_power;
            }
        }

        let average_power = light.power / (gird_count.x * gird_count.y) as f32;
        light.power *= 2.0 * std::f32::consts::PI * std::f32::consts::PI / image.width() as f32;

        light.alias_table = AliasTable::new(&girds_power);
        for power in &mut girds_power {
            if *power > average_power {
                light.skip_compensation = false;
                *power -= average_power;
            } else {
                *power = 0.0;
            }
        }
        if !light.skip_compensation {
            light.alias_table_compensated = AliasTable::new(&girds_power);
        }

        light
    }

    pub(crate) fn power(&self, scene_radius: f32) -> f32 {
        self.power * scene_radius * scene_radius
    }

    pub(crate) fn skip_mis_compensation(&self) -> bool {
        self.skip_compensation
    }

    fn choose_alias_table(&self, mis_compensation: MisCompensation) -> &AliasTable {
        if self.skip_compensation {
            &self.alias_table
        } else {
            match mis_compensation {
                MisCompensation::Disabled => &self.alias_table,
                MisCompensation::Enabled => &self.alias_table_compensated,
            }
        }
    }

    pub(crate) fn sample(
        &self,
        surface_point: glam::Vec3,
        scene_radius: f32,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
        mis_compensation: MisCompensation,
    ) -> Option<LightSample> {
        let sample = self
            .choose_alias_table(mis_compensation)
            .sample(rng.uniform())?;
        let gird_x = sample.index % self.gird_count.x;
        let gird_y = sample.index / self.gird_count.x;
        let w = GIRD_SIZE.min(self.image.width() - gird_x * GIRD_SIZE) as f32;
        let h = GIRD_SIZE.min(self.image.height() - gird_y * GIRD_SIZE) as f32;
        let image_point = glam::vec2(
            (gird_x * GIRD_SIZE) as f32 + w * rng.uniform(),
            (gird_y * GIRD_SIZE) as f32 + h * rng.uniform(),
        );

        let light_direction = self.direction_from_image_point(image_point);
        if light_direction.y.abs() == 1.0 {
            None
        } else {
            Some(LightSample {
                light_point: surface_point + 2.0 * scene_radius * light_direction,
                light_direction,
                radiance: self.image.sample_image_point(image_point, wavelength),
                pdf: SpectrumSample::splat(
                    sample.pmf * self.image.width() as f32 * self.image.height() as f32
                        / (2.0
                            * std::f32::consts::PI
                            * std::f32::consts::PI
                            * (1.0 - light_direction.y * light_direction.y).sqrt()
                            * w
                            * h),
                ),
            })
        }
    }

    pub(crate) fn radiance(
        &self,
        light_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        if light_direction.y.abs() >= 1.0 {
            SpectrumSample::ZERO
        } else {
            let image_point = self.image_point_from_direction(light_direction);
            self.image.sample_image_point(image_point, wavelength)
        }
    }

    pub(crate) fn pdf(
        &self,
        light_direction: glam::Vec3,
        mis_compensation: MisCompensation,
    ) -> SpectrumSample {
        if light_direction.y.abs() >= 1.0 {
            SpectrumSample::ZERO
        } else {
            let image_point = self.image_point_from_direction(light_direction);
            let gird_idx = self.gird_idx_from_image_point(image_point);
            let w = GIRD_SIZE.min(self.image.width() - gird_idx.x * GIRD_SIZE) as f32;
            let h = GIRD_SIZE.min(self.image.height() - gird_idx.y * GIRD_SIZE) as f32;
            let gird_pmf = self
                .choose_alias_table(mis_compensation)
                .pmf(gird_idx.y * self.gird_count.x + gird_idx.x);
            SpectrumSample::splat(
                gird_pmf * self.image.width() as f32 * self.image.height() as f32
                    / (2.0
                        * std::f32::consts::PI
                        * std::f32::consts::PI
                        * (1.0 - light_direction.y * light_direction.y).sqrt()
                        * w
                        * h),
            )
        }
    }

    fn image_point_from_direction(&self, direction: glam::Vec3) -> glam::Vec2 {
        let direction = direction.normalize();
        let theta = direction.y.clamp(-1.0, 1.0).acos().to_degrees();
        let phi = (direction.z.atan2(direction.x).to_degrees() + self.start_phi).rem_euclid(360.0);
        glam::vec2(
            self.image.width() as f32 * (phi / 360.0),
            self.image.height() as f32 * (theta / 180.0),
        )
    }

    fn direction_from_image_point(&self, image_point: glam::Vec2) -> glam::Vec3 {
        let theta = (image_point.y / self.image.height() as f32 * 180.0).to_radians();
        let phi = (image_point.x / self.image.width() as f32 * 360.0 - self.start_phi).to_radians();

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        glam::vec3(sin_phi * cos_phi, cos_theta, sin_theta * sin_phi)
    }

    fn gird_idx_from_image_point(&self, image_point: glam::Vec2) -> glam::USizeVec2 {
        let point_discrete = glam::usizevec2(
            image_point.x.clamp(0.0, self.image.width() as f32) as usize,
            image_point.y.clamp(0.0, self.image.height() as f32) as usize,
        );
        point_discrete / GIRD_SIZE
    }
}
