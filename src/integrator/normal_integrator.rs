use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    color::{LinearRgb, SRGB},
    scene::{HitInfo, Scene},
};

pub struct NormalIntegrator;

impl Integrator for NormalIntegrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        _sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample<'_>> {
        let ray = camera.generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
        if let Some(HitInfo { intersection, .. }) = scene.intersect(&ray, 1e-3, f32::INFINITY) {
            Some(PixelSample::Rgb(
                SRGB.encode(LinearRgb::from_vec3(intersection.normal * 0.5 + 0.5)),
                &SRGB,
            ))
        } else {
            None
        }
    }
}
