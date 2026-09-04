use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    color::ColorSpace,
    geometry::{Frame, Ray},
    light_sampler::{LightSampler, LightSelector, MisCompensation},
    scene::{HitInfo, Scene},
    spectrum::{SpectrumSample, WavelengthSample},
    util::Rng,
};

pub struct SimplePathTracingIntegrator<'a, L> {
    light_sampler: &'a LightSampler<'a, L>,
}

impl<'a, L> SimplePathTracingIntegrator<'a, L> {
    pub fn new(light_sampler: &'a LightSampler<'a, L>) -> Self {
        if let MisCompensation::Enabled = light_sampler.mis_compensation() {
            panic!("SimplePathTracingIntegrator doesn't support MIS Compensation")
        }
        Self { light_sampler }
    }
}

impl<L: LightSelector> Integrator for SimplePathTracingIntegrator<'_, L> {
    fn integrate<'a>(
        &self,
        x: usize,
        y: usize,
        sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
        _target_color_space: &'a ColorSpace,
    ) -> Option<PixelSample<'a>> {
        let mut rng = Rng::new(0, ((x + 1) * (y + 1) * sample_index) as u64);

        let mut wavelength = WavelengthSample::uniform(rng.uniform());
        let mut ray = camera.generate_ray(
            glam::IVec2::new(x as i32, y as i32),
            glam::Vec2::new(rng.uniform(), rng.uniform()),
        );
        let mut beta = SpectrumSample::ONE;
        let mut radiance = SpectrumSample::ZERO;
        let q = 0.9;
        let mut last_is_delta = true;
        let mut last_surface_point = ray.origin;
        let mut terminate_secondary = false;

        loop {
            let Some(HitInfo {
                intersection,
                material,
                area_light,
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            else {
                if last_is_delta {
                    radiance += beta * scene.infinite_radiance(ray.direction, &wavelength);
                }
                break;
            };

            if last_is_delta && let Some(area_light) = area_light {
                radiance += beta
                    * area_light.radiance(
                        last_surface_point,
                        intersection.hit_point,
                        intersection.normal,
                        &wavelength,
                    );
            }
            last_surface_point = intersection.hit_point;

            if rng.uniform() >= q {
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
                && let Some(light_sample) =
                    self.light_sampler
                        .sample_light(intersection.hit_point, &mut rng, &wavelength)
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
                            &wavelength,
                        ) * light_direction_local.y.abs()
                            * light_sample.radiance
                            / light_sample.pdf[0];
                }
            }

            let Some(scattering_sample) = material.bsdf.sample(
                intersection.hit_point,
                view_direction,
                &mut rng,
                &wavelength,
            ) else {
                break;
            };

            if !terminate_secondary && scattering_sample.dispersive_refraction {
                terminate_secondary = true;
            }

            beta *= scattering_sample.bsdf * scattering_sample.light_direction.y.abs()
                / scattering_sample.pdf[0];

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(scattering_sample.light_direction);
        }

        if terminate_secondary {
            wavelength.terminate_secondary();
        }

        Some(PixelSample::Radiance(radiance, wavelength))
    }
}
