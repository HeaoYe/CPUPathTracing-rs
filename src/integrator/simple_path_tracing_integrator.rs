use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    geometry::Frame,
    sample::importance,
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

        loop {
            let Some(HitInfo {
                intersection,
                material,
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            else {
                break;
            };
            radiance += beta * material.emissive;

            if rng.uniform() > q {
                break;
            }

            beta *= material.albedo / q;

            let frame = Frame::new(intersection.normal);
            let light_direction;
            if material.is_specular {
                let view_direction = frame.local_from_world(-ray.direction);
                light_direction =
                    glam::Vec3::new(-view_direction.x, view_direction.y, -view_direction.z);
            } else {
                light_direction = importance::cosine_hemisphere(rng.uniform(), rng.uniform());
                beta *= light_direction.y
                    / (std::f32::consts::PI * importance::cosine_hemisphere_pdf(light_direction));
            }

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(light_direction);
        }

        Some(PixelSample::Radiance(radiance))
    }
}
