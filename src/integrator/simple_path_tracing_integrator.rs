use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    geometry::{Frame, Ray},
    light_sampler::{LightSampler, LightSelector},
    scene::{HitInfo, Scene},
    util::Rng,
};

pub struct SimplePathTracingIntegrator<'a, L> {
    light_sampler: &'a LightSampler<'a, L>,
}

impl<'a, L> SimplePathTracingIntegrator<'a, L> {
    pub fn new(light_sampler: &'a LightSampler<'a, L>) -> Self {
        Self { light_sampler }
    }
}

impl<L: LightSelector> Integrator for SimplePathTracingIntegrator<'_, L> {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample> {
        let mut rng = Rng::new(0, ((x + 1) * (y + 1) * sample_index) as u64);

        let mut ray = camera.generate_ray(
            glam::IVec2::new(x as i32, y as i32),
            glam::Vec2::new(rng.uniform(), rng.uniform()),
        );
        let mut beta = glam::Vec3::ONE;
        let mut radiance = glam::Vec3::ZERO;
        let q = 0.9;
        let mut last_is_delta = true;
        let mut last_surface_point = ray.origin;

        loop {
            let Some(HitInfo {
                intersection,
                material,
                area_light,
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            else {
                if last_is_delta {
                    radiance += beta * scene.infinite_radiance(ray.direction);
                }
                break;
            };

            if last_is_delta && let Some(area_light) = area_light {
                radiance += beta
                    * area_light.radiance(
                        last_surface_point,
                        intersection.hit_point,
                        intersection.normal,
                    );
            }
            last_surface_point = intersection.hit_point;

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

            last_is_delta = material.bsdf.is_delta_distribution();
            if !last_is_delta
                && let Some(light_sample) = self
                    .light_sampler
                    .sample_light(intersection.hit_point, &mut rng)
            {
                let shadow_ray = Ray::new(
                    intersection.hit_point,
                    light_sample.light_point - intersection.hit_point,
                );
                if scene.intersect(&shadow_ray, 1e-4, 1.0 - 1e-4).is_none() {
                    let light_direction_local =
                        frame.local_from_world(light_sample.light_direction);
                    radiance +=
                        beta * material.bsdf.bsdf(
                            intersection.hit_point,
                            light_direction_local,
                            view_direction,
                        ) * light_direction_local.y.abs()
                            * light_sample.radiance
                            / light_sample.pdf;
                }
            }

            let Some(scattering_sample) =
                material
                    .bsdf
                    .sample(intersection.hit_point, view_direction, &mut rng)
            else {
                break;
            };
            beta *= scattering_sample.bsdf * scattering_sample.light_direction.y.abs()
                / scattering_sample.pdf;

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(scattering_sample.light_direction);
        }

        Some(PixelSample::Radiance(radiance))
    }
}
