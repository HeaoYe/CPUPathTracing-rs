use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    geometry::Intersection,
    scene::{HitInfo, Scene},
    util::Rgb,
};

pub struct BoundsTestIntegrator;
pub struct TriangleTestIntegrator;
pub struct BvhDepthIntegrator;

impl Integrator for BoundsTestIntegrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        _sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample> {
        #[cfg(debug_assertions)]
        {
            let ray =
                camera.generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.bounds_test_count as f32 / 200.0,
                )));
            }
        }
        None
    }
}

impl Integrator for TriangleTestIntegrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        _sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample> {
        #[cfg(debug_assertions)]
        {
            let ray =
                camera.generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.triangle_test_count as f32 / 20.0,
                )));
            }
        }
        None
    }
}

impl Integrator for BvhDepthIntegrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        _sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample> {
        #[cfg(debug_assertions)]
        {
            let ray =
                camera.generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(HitInfo {
                intersection: Intersection { debug_info, .. },
                ..
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            {
                return Some(PixelSample::Rgb(Rgb::generate_heatmap_rgb(
                    debug_info.bvh_depth as f32 / 32.0,
                )));
            }
        }
        None
    }
}
