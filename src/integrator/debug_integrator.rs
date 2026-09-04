use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    scene::Scene,
};

pub struct BoundsTestIntegrator;
pub struct PrimitiveTestIntegrator;

impl Integrator for BoundsTestIntegrator {
    fn integrate(
        &self,
        _x: usize,
        _y: usize,
        _sample_index: usize,
        _camera: &CameraModel,
        _scene: &Scene,
    ) -> Option<PixelSample<'_>> {
        #[cfg(debug_assertions)]
        {
            use crate::color::{EncodedRgb, SRGB};
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            _scene.intersect(&ray, 1e-3, f32::INFINITY);
            return Some(PixelSample::Rgb(
                EncodedRgb::generate_heatmap(
                    ray.debug_info.borrow().bounds_test_count as f32 / 150.0,
                ),
                &SRGB,
            ));
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

impl Integrator for PrimitiveTestIntegrator {
    fn integrate(
        &self,
        _x: usize,
        _y: usize,
        _sample_index: usize,
        _camera: &CameraModel,
        _scene: &Scene,
    ) -> Option<PixelSample<'_>> {
        #[cfg(debug_assertions)]
        {
            use crate::color::{EncodedRgb, SRGB};
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            _scene.intersect(&ray, 1e-3, f32::INFINITY);
            return Some(PixelSample::Rgb(
                EncodedRgb::generate_heatmap(
                    ray.debug_info.borrow().primitive_test_count as f32 / 14.0,
                ),
                &SRGB,
            ));
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}
