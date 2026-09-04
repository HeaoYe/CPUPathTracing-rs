use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    color::ColorSpace,
    geometry::{Frame, Ray},
    light_sampler::{LightSampler, LightSelector},
    scene::{HitInfo, Scene},
    spectrum::{SpectrumSample, WavelengthSample},
    util::Rng,
};

pub struct PathTracingIntegrator<'a, L> {
    light_sampler: &'a LightSampler<'a, L>,
}

impl<'a, L> PathTracingIntegrator<'a, L> {
    pub fn new(light_sampler: &'a LightSampler<'a, L>) -> Self {
        Self { light_sampler }
    }
}

fn power_heuristic(pdf_a: f32, pdf_b: f32) -> f32 {
    pdf_a * pdf_a / (pdf_a * pdf_a + pdf_b * pdf_b)
}

impl<L: LightSelector> Integrator for PathTracingIntegrator<'_, L> {
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
        let mut last_is_delta = true;
        let mut last_surface_point = ray.origin;
        let mut last_pdf_bsdf = 0.0;
        let mut eta_scale = SpectrumSample::ONE;
        let mut terminate_secondary = false;

        loop {
            let Some(HitInfo {
                intersection,
                material,
                area_light,
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            else {
                let light_direction = ray.direction.normalize();
                if last_is_delta {
                    radiance += beta * scene.infinite_radiance(light_direction, &wavelength);
                } else {
                    let light_point = ray.origin + 2.0 * scene.radius() * ray.direction;
                    for (id, light) in scene.infinite_lights() {
                        let pdf_light =
                            self.light_sampler
                                .pdf(id, ray.origin, light_point, -ray.direction);
                        let weight_bsdf = power_heuristic(last_pdf_bsdf, pdf_light);
                        radiance += SpectrumSample::splat(weight_bsdf)
                            * beta
                            * light.radiance(light_direction, &wavelength);
                    }
                }
                break;
            };

            if let Some(area_light) = area_light {
                let weight_bsdf = if last_is_delta {
                    1.0
                } else {
                    let pdf_light = self.light_sampler.pdf(
                        scene
                            .get_area_light_id(area_light.shape_instance_id)
                            .unwrap(),
                        last_surface_point,
                        intersection.hit_point,
                        intersection.normal,
                    );
                    power_heuristic(last_pdf_bsdf, pdf_light)
                };
                radiance += SpectrumSample::splat(weight_bsdf)
                    * beta
                    * area_light.radiance(
                        last_surface_point,
                        intersection.hit_point,
                        intersection.normal,
                        &wavelength,
                    );
            }
            last_surface_point = intersection.hit_point;

            let beta_q = beta * eta_scale;
            let q = beta_q.max_element().min(0.9);
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
                    let pdf_bsdf =
                        material
                            .bsdf
                            .pdf(light_direction_local, view_direction, &wavelength);
                    let weight_light = power_heuristic(light_sample.pdf, pdf_bsdf);
                    radiance += SpectrumSample::splat(weight_light)
                        * beta
                        * material.bsdf.bsdf(
                            intersection.hit_point,
                            light_direction_local,
                            view_direction,
                            &wavelength,
                        )
                        * light_direction_local.y.abs()
                        * light_sample.radiance
                        / light_sample.pdf;
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

            last_pdf_bsdf = scattering_sample.pdf;
            eta_scale *= scattering_sample.eta_scale;
            beta *= scattering_sample.bsdf * scattering_sample.light_direction.y.abs()
                / scattering_sample.pdf;

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(scattering_sample.light_direction);
        }

        if terminate_secondary {
            wavelength.terminate_secondary();
        }

        Some(PixelSample::Radiance(radiance, wavelength))
    }
}
