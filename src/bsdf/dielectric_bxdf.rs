use super::{
    bxdf::{Bxdf, ScatteringSample},
    microfacet_theory::MicrofacetTheory,
};

pub struct DielectricBxdf {
    ior: f32,
    reflectance: glam::Vec3,
    transmittance: glam::Vec3,
    microfacet_theory: MicrofacetTheory,
}

impl DielectricBxdf {
    pub fn new(
        ior: f32,
        reflectance: glam::Vec3,
        transmittance: glam::Vec3,
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

impl Bxdf for DielectricBxdf {
    fn sample(
        &self,
        _hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        if self.ior == 1.0 {
            return Some(ScatteringSample {
                bsdf: self.transmittance / view_direction.y.abs(),
                pdf: 1.0,
                light_direction: -view_direction,
            });
        }

        let microfacet_normal = if !self.microfacet_theory.is_delta_distribution() {
            self.microfacet_theory
                .sample_visible_normal(view_direction, rng)
        } else {
            glam::Vec3::Y
        };

        let cos_theta_t = view_direction.dot(microfacet_normal).abs();
        let (etai_div_etat, scale) = if view_direction.y > 0.0 {
            (self.ior, 1.0)
        } else {
            (1.0 / self.ior, -1.0)
        };

        let mut cos_theta_i = 0.0;
        let fr = fresnel(etai_div_etat, cos_theta_t, &mut cos_theta_i);

        if rng.uniform() < fr {
            let light_direction = -view_direction + 2.0 * cos_theta_i * microfacet_normal;
            if light_direction.y * view_direction.y <= 0.0 {
                return None;
            }

            if self.microfacet_theory.is_delta_distribution() {
                return Some(ScatteringSample {
                    bsdf: fr * self.reflectance / light_direction.y.abs(),
                    pdf: fr,
                    light_direction,
                });
            }

            let bsdf = fr
                * self
                    .microfacet_theory
                    .normal_distribution(microfacet_normal)
                * self.microfacet_theory.height_correlated_masking_shadowing(
                    light_direction,
                    view_direction,
                    microfacet_normal,
                )
                / (4.0 * light_direction.y * view_direction.y).abs();
            let pdf = self
                .microfacet_theory
                .visible_normal_distribution(view_direction, microfacet_normal)
                / (4.0 * cos_theta_i).abs();
            Some(ScatteringSample {
                bsdf: glam::Vec3::splat(bsdf),
                pdf,
                light_direction,
            })
        } else {
            let light_direction = -view_direction / etai_div_etat
                + (cos_theta_t / etai_div_etat - cos_theta_i) * scale * microfacet_normal;
            let lv = light_direction.y * view_direction.y;
            if lv >= 0.0 {
                return None;
            }

            if self.microfacet_theory.is_delta_distribution() {
                return Some(ScatteringSample {
                    bsdf: (1.0 - fr) * self.transmittance / light_direction.y.abs(),
                    pdf: 1.0 - fr,
                    light_direction,
                });
            }

            let det_j = etai_div_etat * etai_div_etat * cos_theta_i.abs()
                / (cos_theta_t - etai_div_etat * cos_theta_i.abs()).powi(2);
            let bsdf = (1.0 - fr)
                * self.transmittance
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
                / lv
                / (etai_div_etat * etai_div_etat);
            let pdf = (1.0 - fr)
                * self
                    .microfacet_theory
                    .visible_normal_distribution(view_direction, microfacet_normal)
                * det_j;
            Some(ScatteringSample {
                bsdf,
                pdf,
                light_direction,
            })
        }
    }
}
