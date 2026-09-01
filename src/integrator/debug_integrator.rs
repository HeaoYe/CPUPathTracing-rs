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
    ) -> Option<PixelSample> {
        #[cfg(debug_assertions)]
        {
            use crate::util::Rgb;
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            _scene.intersect(&ray, 1e-3, f32::INFINITY);
            return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                ray.debug_info.borrow().bounds_test_count as f32 / 150.0,
            )));
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
    ) -> Option<PixelSample> {
        #[cfg(debug_assertions)]
        {
            use crate::util::Rgb;
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            _scene.intersect(&ray, 1e-3, f32::INFINITY);
            return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                ray.debug_info.borrow().primitive_test_count as f32 / 14.0,
            )));
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}
