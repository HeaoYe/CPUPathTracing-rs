use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    scene::Scene,
};

pub struct BoundsTestIntegrator;
pub struct TriangleTestIntegrator;
pub struct BvhDepthIntegrator;

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
            use crate::{geometry::Intersection, scene::HitInfo, util::Rgb};
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = _scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.bounds_test_count as f32 / 150.0,
                )));
            }
        }
        None
    }
}

impl Integrator for TriangleTestIntegrator {
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
            use crate::{geometry::Intersection, scene::HitInfo, util::Rgb};
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = _scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.triangle_test_count as f32 / 7.0,
                )));
            }
        }
        None
    }
}

impl Integrator for BvhDepthIntegrator {
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
            use crate::{geometry::Intersection, scene::HitInfo, util::Rgb};
            let ray = _camera.generate_ray(
                glam::IVec2::new(_x as i32, _y as i32),
                glam::Vec2::splat(0.5),
            );
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = _scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.bvh_depth as f32 / 32.0,
                )));
            }
        }
        None
    }
}
