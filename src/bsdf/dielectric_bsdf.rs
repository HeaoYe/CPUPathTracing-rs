use super::{ScatteringSample, microfacet_theory::MicrofacetTheory};
use crate::spectrum::{Spectrum, SpectrumSample, WAVELENGTH_SAMPLE_COUNT, WavelengthSample};

#[derive(Clone, Copy)]
pub struct DielectricBsdf<'a> {
    ior: &'a Spectrum<'a>,
    reflectance: &'a Spectrum<'a>,
    transmittance: &'a Spectrum<'a>,
    microfacet_theory: MicrofacetTheory,
}

impl<'a> DielectricBsdf<'a> {
    pub fn new(
        ior: &'a Spectrum<'a>,
        reflectance: &'a Spectrum<'a>,
        transmittance: &'a Spectrum<'a>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
            microfacet_theory: MicrofacetTheory::new(alpha_x, alpha_z),
        }
    }
}

fn fresnel(etai_div_etat: f32, cos_theta_t: f32, cos_theta_i: &mut f32) -> f32 {
    let cos_theta_t = cos_theta_t.clamp(0.0, 1.0);
    let sin2_theta_t = (1.0 - cos_theta_t * cos_theta_t).clamp(0.0, 1.0);
    let sin2_theta_i = sin2_theta_t / (etai_div_etat * etai_div_etat);

    if sin2_theta_i >= 1.0 {
        return 1.0;
    }

    *cos_theta_i = (1.0 - sin2_theta_i).sqrt();
    let r_parl =
        (*cos_theta_i - etai_div_etat * cos_theta_t) / (*cos_theta_i + etai_div_etat * cos_theta_t);
    let r_perp =
        (etai_div_etat * *cos_theta_i - cos_theta_t) / (etai_div_etat * *cos_theta_i + cos_theta_t);
    0.5 * (r_parl * r_parl + r_perp * r_perp)
}

fn fresnel_spectrum(
    etai_div_etat: SpectrumSample,
    cos_theta_t: f32,
    cos_theta_i: &mut SpectrumSample,
) -> SpectrumSample {
    let mut result = SpectrumSample::default();

    let cos_theta_t = cos_theta_t.clamp(0.0, 1.0);
    let sin2_theta_t = (1.0 - cos_theta_t * cos_theta_t).clamp(0.0, 1.0);
    let sin2_theta_i = SpectrumSample::splat(sin2_theta_t) / (etai_div_etat * etai_div_etat);

    for i in 0..WAVELENGTH_SAMPLE_COUNT {
        if sin2_theta_i[i] >= 1.0 {
            result[i] = 1.0;
            continue;
        }

        cos_theta_i[i] = (1.0 - sin2_theta_i[i]).sqrt();
        let r_parl = (cos_theta_i[i] - etai_div_etat[i] * cos_theta_t)
            / (cos_theta_i[i] + etai_div_etat[i] * cos_theta_t);
        let r_perp = (etai_div_etat[i] * cos_theta_i[i] - cos_theta_t)
            / (etai_div_etat[i] * cos_theta_i[i] + cos_theta_t);
        result[i] = 0.5 * (r_parl * r_parl + r_perp * r_perp);
    }

    result
}

impl DielectricBsdf<'_> {
    pub(super) fn is_delta_distribution(&self) -> bool {
        (self.ior.is_constant() && self.ior.max() == 1.0)
            || self.microfacet_theory.is_delta_distribution()
    }

    pub(super) fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
        wavelength: &WavelengthSample,
    ) -> Option<ScatteringSample> {
        if self.ior.is_constant() && self.ior.max() == 1.0 {
            return Some(ScatteringSample::new(
                self.transmittance.sample(wavelength) / view_direction.y.abs(),
                SpectrumSample::ONE,
                -view_direction,
            ));
        }

        let microfacet_normal = if !self.microfacet_theory.is_delta_distribution() {
            self.microfacet_theory
                .sample_visible_normal(view_direction, rng)
        } else {
            glam::Vec3::Y
        };

        let cos_theta_t = view_direction.dot(microfacet_normal).abs();
        let (etai_div_etat, scale) = if view_direction.y > 0.0 {
            (self.ior.sample(wavelength), 1.0)
        } else {
            (SpectrumSample::ONE / self.ior.sample(wavelength), -1.0)
        };

        let mut cos_theta_i = SpectrumSample::ZERO;
        let fr = fresnel_spectrum(etai_div_etat, cos_theta_t, &mut cos_theta_i);

        if rng.uniform() < fr[0] {
            let light_direction =
                -view_direction + 2.0 * view_direction.dot(microfacet_normal) * microfacet_normal;
            if light_direction.y * view_direction.y <= 0.0 {
                return None;
            }

            if self.microfacet_theory.is_delta_distribution() {
                return Some(ScatteringSample::new(
                    fr * self.reflectance.sample(wavelength) / light_direction.y.abs(),
                    fr,
                    light_direction,
                ));
            }

            let bsdf = fr
                * self.reflectance.sample(wavelength)
                * self
                    .microfacet_theory
                    .normal_distribution(microfacet_normal)
                * self.microfacet_theory.height_correlated_masking_shadowing(
                    light_direction,
                    view_direction,
                    microfacet_normal,
                )
                / (4.0 * light_direction.y * view_direction.y).abs();
            let pdf = fr
                * self
                    .microfacet_theory
                    .visible_normal_distribution(view_direction, microfacet_normal)
                / (4.0 * cos_theta_t);
            Some(ScatteringSample::new(bsdf, pdf, light_direction))
        } else {
            let light_direction = -view_direction / etai_div_etat[0]
                + (cos_theta_t / etai_div_etat[0] - cos_theta_i[0]) * scale * microfacet_normal;
            let lv = light_direction.y * view_direction.y;
            if lv >= 0.0 {
                return None;
            }

            let eta_scale = etai_div_etat * etai_div_etat;

            if self.microfacet_theory.is_delta_distribution() {
                if self.ior.is_constant() {
                    return Some(
                        ScatteringSample::new(
                            (SpectrumSample::ONE - fr) * self.transmittance.sample(wavelength)
                                / light_direction.y.abs()
                                / eta_scale,
                            SpectrumSample::splat(1.0 - fr[0]),
                            light_direction,
                        )
                        .with_eta_scale(eta_scale),
                    );
                }
                let mut bsdf = SpectrumSample::ZERO;
                bsdf[0] = (1.0 - fr[0]) * self.transmittance.eval(wavelength.lambda(0))
                    / light_direction.y.abs()
                    / eta_scale[0];
                let mut pdf = SpectrumSample::ZERO;
                pdf[0] = 1.0 - fr[0];
                return Some(
                    ScatteringSample::new(bsdf, pdf, light_direction)
                        .with_eta_scale(eta_scale)
                        .with_dispersive_refraction(),
                );
            }

            let mut sample = ScatteringSample::new(
                self.bsdf(light_direction, view_direction, wavelength),
                self.pdf(light_direction, view_direction, wavelength),
                light_direction,
            )
            .with_eta_scale(eta_scale);
            if !self.ior.is_constant() {
                sample = sample.with_dispersive_refraction();
            }
            Some(sample)
        }
    }

    pub(super) fn bsdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        if self.is_delta_distribution() {
            return SpectrumSample::ZERO;
        }

        let lv = light_direction.y * view_direction.y;
        if lv == 0.0 {
            return SpectrumSample::ZERO;
        }

        let etai_div_etat = if view_direction.y > 0.0 {
            self.ior.sample(wavelength)
        } else {
            SpectrumSample::ONE / self.ior.sample(wavelength)
        };

        if lv > 0.0 {
            let mut microfacet_normal = light_direction + view_direction;
            if microfacet_normal.length_squared() == 0.0 {
                return SpectrumSample::ZERO;
            }
            if microfacet_normal.y < 0.0 {
                microfacet_normal = -microfacet_normal;
            }
            if light_direction.dot(microfacet_normal) * light_direction.y <= 0.0
                || view_direction.dot(microfacet_normal) * view_direction.y <= 0.0
            {
                return SpectrumSample::ZERO;
            }
            microfacet_normal = microfacet_normal.normalize();

            let cos_theta_t = view_direction.dot(microfacet_normal).abs();
            let mut cos_theta_i = SpectrumSample::ZERO;
            let fr = fresnel_spectrum(etai_div_etat, cos_theta_t, &mut cos_theta_i);

            return fr
                * self.reflectance.sample(wavelength)
                * self
                    .microfacet_theory
                    .normal_distribution(microfacet_normal)
                * self.microfacet_theory.height_correlated_masking_shadowing(
                    light_direction,
                    view_direction,
                    microfacet_normal,
                )
                / (4.0 * lv);
        }

        let eta_scale = etai_div_etat * etai_div_etat;
        let mut bsdf = self.transmittance.sample(wavelength);
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            let mut microfacet_normal = light_direction + view_direction / etai_div_etat[i];
            if microfacet_normal.length_squared() == 0.0 {
                bsdf[i] = 0.0;
                continue;
            }
            if microfacet_normal.y < 0.0 {
                microfacet_normal = -microfacet_normal;
            }
            if light_direction.dot(microfacet_normal) * light_direction.y <= 0.0
                || view_direction.dot(microfacet_normal) * view_direction.y <= 0.0
            {
                bsdf[i] = 0.0;
                continue;
            }
            microfacet_normal = microfacet_normal.normalize();

            let cos_theta_t = view_direction.dot(microfacet_normal).abs();
            let mut cos_theta_i = 0.0;
            let fr = fresnel(etai_div_etat[i], cos_theta_t, &mut cos_theta_i);
            if fr == 1.0 {
                bsdf[i] = 0.0;
                continue;
            }

            let det_j = eta_scale[i] * cos_theta_i.abs()
                / (cos_theta_t - etai_div_etat[i] * cos_theta_i.abs()).powi(2);
            bsdf[i] *= (1.0 - fr)
                * det_j
                * self
                    .microfacet_theory
                    .normal_distribution(microfacet_normal)
                * self.microfacet_theory.height_correlated_masking_shadowing(
                    light_direction,
                    view_direction,
                    microfacet_normal,
                )
                * cos_theta_t
                / lv.abs()
                / eta_scale[i];
        }

        bsdf
    }

    pub(super) fn pdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        wavelength: &WavelengthSample,
    ) -> SpectrumSample {
        if self.is_delta_distribution() {
            return SpectrumSample::ZERO;
        }

        let lv = light_direction.y * view_direction.y;
        if lv == 0.0 {
            return SpectrumSample::ZERO;
        }

        let etai_div_etat = if view_direction.y > 0.0 {
            self.ior.sample(wavelength)
        } else {
            SpectrumSample::ONE / self.ior.sample(wavelength)
        };

        if lv > 0.0 {
            let mut microfacet_normal = light_direction + view_direction;
            if microfacet_normal.length_squared() == 0.0 {
                return SpectrumSample::ZERO;
            }
            if microfacet_normal.y < 0.0 {
                microfacet_normal = -microfacet_normal;
            }
            if light_direction.dot(microfacet_normal) * light_direction.y <= 0.0
                || view_direction.dot(microfacet_normal) * view_direction.y <= 0.0
            {
                return SpectrumSample::ZERO;
            }
            microfacet_normal = microfacet_normal.normalize();

            let cos_theta_t = view_direction.dot(microfacet_normal).abs();
            let mut cos_theta_i = SpectrumSample::ZERO;
            let fr = fresnel_spectrum(etai_div_etat, cos_theta_t, &mut cos_theta_i);

            return fr
                * self
                    .microfacet_theory
                    .visible_normal_distribution(view_direction, microfacet_normal)
                / (4.0 * cos_theta_t);
        }

        let mut pdf = SpectrumSample::ZERO;
        for i in 0..WAVELENGTH_SAMPLE_COUNT {
            let mut microfacet_normal = light_direction + view_direction / etai_div_etat[i];
            if microfacet_normal.length_squared() == 0.0 {
                continue;
            }
            if microfacet_normal.y < 0.0 {
                microfacet_normal = -microfacet_normal;
            }
            if light_direction.dot(microfacet_normal) * light_direction.y <= 0.0
                || view_direction.dot(microfacet_normal) * view_direction.y <= 0.0
            {
                continue;
            }
            microfacet_normal = microfacet_normal.normalize();

            let cos_theta_t = view_direction.dot(microfacet_normal).abs();
            let mut cos_theta_i = 0.0;
            let fr = fresnel(etai_div_etat[i], cos_theta_t, &mut cos_theta_i);
            if fr == 1.0 {
                continue;
            }

            let det_j = etai_div_etat[i] * etai_div_etat[i] * cos_theta_i.abs()
                / (cos_theta_t - etai_div_etat[i] * cos_theta_i.abs()).powi(2);
            pdf[i] = (1.0 - fr)
                * self
                    .microfacet_theory
                    .visible_normal_distribution(view_direction, microfacet_normal)
                * det_j;
        }

        pdf
    }
}
