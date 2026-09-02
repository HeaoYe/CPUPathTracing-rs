use super::Integrator;
use crate::{
    bsdf::Bxdf,
    camera::{CameraModel, PixelSample},
    geometry::Frame,
    scene::{HitInfo, Scene},
    util::Rng,
};

pub struct SimplePathTracingIntegrator;

impl Integrator for SimplePathTracingIntegrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample> {
        let mut rng = Rng::new(0, ((x + 1) * (y + 1) * (sample_index + 1)) as u64);

        let mut ray = camera.generate_ray(
            glam::IVec2::new(x as i32, y as i32),
            glam::Vec2::new(rng.uniform(), rng.uniform()),
        );
        let mut beta = glam::Vec3::ONE;
        let mut radiance = glam::Vec3::ZERO;
        let q = 0.9;

        while let Some(HitInfo {
            intersection,
            material,
        }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
        {
            radiance += beta * material.emissive;

            if rng.uniform() > q {
                break;
            }

            beta /= q;

            let frame = Frame::new(intersection.normal);
            let view_direction = frame.local_from_world(-ray.direction);

            if view_direction.y == 0.0 {
                ray.origin = intersection.hit_point;
                continue;
            }

            let Some(scattering_sample) = material.bsdf.sample(view_direction, &mut rng) else {
                break;
            };
            let light_direction = scattering_sample.light_direction;
            beta *= scattering_sample.bsdf * light_direction.y.abs() / scattering_sample.pdf;

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(light_direction);
        }

        Some(PixelSample::Radiance(radiance))
    }
}
