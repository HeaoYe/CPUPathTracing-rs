use super::{ScatteringSample, microfacet_theory::MicrofacetTheory};
use crate::util::Complex;

pub struct ConductorBsdf {
    eta: glam::Vec3,
    k: glam::Vec3,
    microfacet_theory: MicrofacetTheory,
}

impl ConductorBsdf {
    pub fn new(eta: glam::Vec3, k: glam::Vec3, alpha_x: f32, alpha_z: f32) -> Self {
        Self {
            eta,
            k,
            microfacet_theory: MicrofacetTheory::new(alpha_x, alpha_z),
        }
    }
}

fn fresnel(eta: glam::Vec3, k: glam::Vec3, cos_theta_i: f32) -> glam::Vec3 {
    let mut fr = glam::Vec3::ZERO;
    let cos_theta_i = cos_theta_i.clamp(0.0, 1.0);
    for i in 0..3 {
        let etat_div_etai = Complex::new(eta[i], k[i]);
        let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
        let sin2_theta_t = Complex::from(sin2_theta_i) / (etat_div_etai * etat_div_etai);
        let cos_theta_t = (Complex::from(1.0) - sin2_theta_t).sqrt();

        let r_parl = (etat_div_etai * cos_theta_i - cos_theta_t)
            / (etat_div_etai * cos_theta_i + cos_theta_t);
        let r_perp = (Complex::from(cos_theta_i) - etat_div_etai * cos_theta_t)
            / (Complex::from(cos_theta_i) + etat_div_etai * cos_theta_t);

        fr[i] = 0.5 * (r_parl.norm_squared() + r_perp.norm_squared());
    }
    fr
}

impl ConductorBsdf {
    pub(super) fn is_delta_distribution(&self) -> bool {
        self.microfacet_theory.is_delta_distribution()
    }

    pub(super) fn sample(
        &self,
        view_direction: glam::Vec3,
        rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        if view_direction.y <= 0.0 {
            return None;
        }

        let microfacet_normal = if !self.microfacet_theory.is_delta_distribution() {
            self.microfacet_theory
                .sample_visible_normal(view_direction, rng)
        } else {
            glam::Vec3::Y
        };

        let cos_theta_i = view_direction.dot(microfacet_normal);

        let fr = fresnel(self.eta, self.k, cos_theta_i.abs());
        let light_direction = -view_direction + 2.0 * cos_theta_i * microfacet_normal;
        if light_direction.y * view_direction.y <= 0.0 {
            return None;
        }

        if self.microfacet_theory.is_delta_distribution() {
            return Some(ScatteringSample::new(
                fr / light_direction.y.abs(),
                1.0,
                light_direction,
            ));
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
        Some(ScatteringSample::new(bsdf, pdf, light_direction))
    }

    pub(super) fn bsdf(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
    ) -> glam::Vec3 {
        if self.is_delta_distribution() {
            return glam::Vec3::ZERO;
        }

        if view_direction.y <= 0.0 || light_direction.y <= 0.0 {
            return glam::Vec3::ZERO;
        }

        let mut microfacet_normal = (light_direction + view_direction).normalize();
        if microfacet_normal.y < 0.0 {
            microfacet_normal = -microfacet_normal;
        }
        let cos_theta_i = view_direction.dot(microfacet_normal).abs();
        let fr = fresnel(self.eta, self.k, cos_theta_i);
        fr * self
            .microfacet_theory
            .normal_distribution(microfacet_normal)
            * self.microfacet_theory.height_correlated_masking_shadowing(
                light_direction,
                view_direction,
                microfacet_normal,
            )
            / (4.0 * light_direction.y * view_direction.y)
    }

    pub(super) fn pdf(&self, light_direction: glam::Vec3, view_direction: glam::Vec3) -> f32 {
        if self.is_delta_distribution() {
            return 0.0;
        }

        if view_direction.y <= 0.0 || light_direction.y <= 0.0 {
            return 0.0;
        }

        let mut microfacet_normal = (light_direction + view_direction).normalize();
        if microfacet_normal.y < 0.0 {
            microfacet_normal = -microfacet_normal;
        }
        self.microfacet_theory
            .visible_normal_distribution(view_direction, microfacet_normal)
            / (4.0 * view_direction.dot(microfacet_normal).abs())
    }
}
