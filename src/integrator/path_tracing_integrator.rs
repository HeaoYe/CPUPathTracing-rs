use super::Integrator;
use crate::{
    camera::{CameraModel, PixelSample},
    color::ColorSpace,
    geometry::{Frame, Ray},
    light_sampler::{LightSampler, LightSelector},
    scene::{HitInfo, Scene},
    spectrum::{SpectrumSample, WAVELENGTH_SAMPLE_COUNT, WavelengthSample},
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

enum SampleTech {
    Light,
    Bsdf,
}

fn power_heuristic(
    tech: SampleTech,
    log_q_light: SpectrumSample,
    log_q_bsdf: SpectrumSample,
) -> f32 {
    let mut denom = 0.0;
    let max_log = log_q_light.max_element().max(log_q_bsdf.max_element());
    for i in 0..WAVELENGTH_SAMPLE_COUNT {
        denom += (2.0 * (log_q_light[i] - max_log)).exp() + (2.0 * (log_q_bsdf[i] - max_log)).exp();
    }
    match tech {
        SampleTech::Light => (2.0 * (log_q_light[0] - max_log)).exp() / denom,
        SampleTech::Bsdf => (2.0 * (log_q_bsdf[0] - max_log)).exp() / denom,
    }
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

        let wavelength = WavelengthSample::uniform(rng.uniform());
        let mut ray = camera.generate_ray(
            glam::IVec2::new(x as i32, y as i32),
            glam::Vec2::new(rng.uniform(), rng.uniform()),
        );
        let mut depth = 0usize;
        let mut beta = SpectrumSample::ONE;
        let mut log_q_pre = SpectrumSample::ONE;
        let mut last_log_q_pre = SpectrumSample::ONE;
        let mut radiance = SpectrumSample::ZERO;
        let mut last_is_delta = true;
        let mut last_surface_point = ray.origin;
        let mut eta_scale = SpectrumSample::ONE;

        loop {
            depth += 1;

            let Some(HitInfo {
                intersection,
                material,
                area_light,
            }) = scene.intersect(&ray, 1e-3, f32::INFINITY)
            else {
                let light_direction = ray.direction.normalize();
                let light_point = ray.origin + 2.0 * scene.radius() * ray.direction;
                if depth == 1 {
                    radiance += scene.infinite_radiance(light_direction, &wavelength);
                } else {
                    if last_is_delta {
                        for (_, light) in scene.infinite_lights() {
                            let weight_bsdf = power_heuristic(
                                SampleTech::Bsdf,
                                SpectrumSample::splat(f32::NEG_INFINITY),
                                log_q_pre,
                            );
                            radiance +=
                                SpectrumSample::splat(weight_bsdf * WAVELENGTH_SAMPLE_COUNT as f32)
                                    * beta
                                    * light.radiance(light_direction, &wavelength);
                        }
                    } else {
                        for (id, light) in scene.infinite_lights() {
                            let pdf_light = self.light_sampler.pdf(
                                id,
                                last_surface_point,
                                light_point,
                                -light_direction,
                            );
                            let log_q_light = last_log_q_pre + pdf_light.ln();
                            let weight_bsdf =
                                power_heuristic(SampleTech::Bsdf, log_q_light, log_q_pre);
                            radiance +=
                                SpectrumSample::splat(weight_bsdf * WAVELENGTH_SAMPLE_COUNT as f32)
                                    * beta
                                    * light.radiance(light_direction, &wavelength);
                        }
                    }
                }
                break;
            };

            if let Some(area_light) = area_light {
                let emission = area_light.radiance(
                    last_surface_point,
                    intersection.hit_point,
                    intersection.normal,
                    &wavelength,
                );
                if depth == 1 {
                    radiance += emission;
                } else {
                    let weight_bsdf = if last_is_delta {
                        power_heuristic(
                            SampleTech::Bsdf,
                            SpectrumSample::splat(f32::NEG_INFINITY),
                            log_q_pre,
                        )
                    } else {
                        let pdf_light = self.light_sampler.pdf(
                            scene
                                .get_area_light_id(area_light.shape_instance_id)
                                .unwrap(),
                            last_surface_point,
                            intersection.hit_point,
                            intersection.normal,
                        );
                        let log_q_light = last_log_q_pre + pdf_light.ln();
                        power_heuristic(SampleTech::Bsdf, log_q_light, log_q_pre)
                    };
                    radiance += SpectrumSample::splat(weight_bsdf * WAVELENGTH_SAMPLE_COUNT as f32)
                        * beta
                        * emission;
                }
            }
            last_surface_point = intersection.hit_point;

            if depth > 3 {
                let mut q = SpectrumSample::ZERO;
                for i in 0..WAVELENGTH_SAMPLE_COUNT {
                    let beta_q = beta * (log_q_pre[0] - log_q_pre[i]).exp() * eta_scale;
                    q[i] = beta_q.max_element().min(0.9);
                }
                if rng.uniform() >= q[0] {
                    break;
                }
                beta /= q[0];
                log_q_pre += q.ln();
            }

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
                    let log_q_light = log_q_pre + light_sample.pdf.ln();
                    let pdf_bsdf =
                        material
                            .bsdf
                            .pdf(light_direction_local, view_direction, &wavelength);
                    let weight_light =
                        power_heuristic(SampleTech::Light, log_q_light, log_q_pre + pdf_bsdf.ln());
                    radiance += SpectrumSample::splat(weight_light * WAVELENGTH_SAMPLE_COUNT as f32)
                        * beta
                        * material.bsdf.bsdf(
                            intersection.hit_point,
                            light_direction_local,
                            view_direction,
                            &wavelength,
                        )
                        * light_direction_local.y.abs()
                        * light_sample.radiance
                        / light_sample.pdf[0]
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

            last_log_q_pre = log_q_pre;
            eta_scale *= scattering_sample.eta_scale;
            beta *= scattering_sample.bsdf * scattering_sample.light_direction.y.abs()
                / scattering_sample.pdf[0];
            log_q_pre += scattering_sample.pdf.ln();

            ray.origin = intersection.hit_point;
            ray.direction = frame.world_from_local(scattering_sample.light_direction);
        }

        Some(PixelSample::Radiance(radiance, wavelength))
    }
}
