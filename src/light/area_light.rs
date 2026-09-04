use super::LightSample;
use crate::{
    geometry::{Sampleable, SurfaceSample},
    scene::ShapeInstanceId,
    spectrum::{Spectrum, SpectrumSample, WavelengthSample},
    util::Rng,
};

pub struct AreaLight<'a> {
    pub(crate) shape_instance_id: ShapeInstanceId,
    radiance: &'a Spectrum<'a>,
    double_side: bool,
}

impl<'a> AreaLight<'a> {
    pub(crate) fn new(
        shape_instance_id: ShapeInstanceId,
        radiance: &'a Spectrum<'a>,
        double_side: bool,
    ) -> Self {
        Self {
            shape_instance_id,
            radiance,
            double_side,
        }
    }

    pub(crate) fn power(&self, shape: &dyn Sampleable) -> f32 {
        (if self.double_side { 2.0 } else { 1.0 })
            * std::f32::consts::PI
            * shape.area()
            * self.radiance.max()
    }

    pub(crate) fn sample(
        &self,
        surface_point: glam::Vec3,
        shape: &dyn Sampleable,
        world_from_object: &glam::Affine3A,
        object_from_world: &glam::Affine3A,
        rng: &mut Rng,
        wavelength: &WavelengthSample,
    ) -> Option<LightSample> {
        let SurfaceSample {
            position,
            normal,
            pdf,
        } = shape.sample(rng)?;
        let light_point = world_from_object.transform_point3(position);
        let normal = object_from_world.matrix3.mul_transpose_vec3(normal);

        let light_direction_raw = light_point - surface_point;
        let light_direction = light_direction_raw.normalize();
        let cos_theta = normal.dot(-light_direction);
        if cos_theta == 0.0 {
            return None;
        }
        if !self.double_side && cos_theta < 0.0 {
            return None;
        }
        let det_j = cos_theta.abs() / light_direction_raw.length_squared();
        Some(LightSample {
            light_point,
            light_direction,
            radiance: self.radiance.sample(wavelength),
            pdf: SpectrumSample::splat(pdf / det_j),
        })
    }

    pub(crate) fn radiance(
        &self,
        surface_point: glam::Vec3,
        light_point: glam::Vec3,
        normal: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        let cos_theta_l = normal.dot(surface_point - light_point);
        if cos_theta_l == 0.0 {
            return SpectrumSample::ZERO;
        }
        if !self.double_side && cos_theta_l < 0.0 {
            return SpectrumSample::ZERO;
        }
        self.radiance.sample(wavelength)
    }

    pub(crate) fn pdf(
        &self,
        shape: &dyn Sampleable,
        surface_point: glam::Vec3,
        light_point: glam::Vec3,
        normal: glam::Vec3,
    ) -> SpectrumSample {
        let cos_theta = (surface_point - light_point).normalize().dot(normal);
        if cos_theta == 0.0 {
            return SpectrumSample::ZERO;
        }
        if !self.double_side && cos_theta < 0.0 {
            return SpectrumSample::ZERO;
        }
        let det_j = cos_theta.abs() / (surface_point - light_point).length_squared();
        SpectrumSample::splat(shape.pdf() / det_j)
    }
}
