use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    color::{ColorSpace, LinearRgb},
    scene::{HitInfo, Scene},
};

pub struct NormalIntegrator;

impl Integrator for NormalIntegrator {
    fn integrate<'a>(
        &self,
        x: usize,
        y: usize,
        _sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
        target_color_space: &'a ColorSpace,
    ) -> Option<PixelSample<'a>> {
        let ray = camera.generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
        if let Some(HitInfo { intersection, .. }) = scene.intersect(&ray, 1e-3, f32::INFINITY) {
            Some(PixelSample::Rgb(
                target_color_space.encode(LinearRgb::from_vec3(intersection.normal * 0.5 + 0.5)),
                target_color_space,
            ))
        } else {
            None
        }
    }
}
