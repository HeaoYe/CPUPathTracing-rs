use super::bxdf::{Bxdf, ScatteringSample};
use crate::util::Complex;

pub struct ConductorBxdf {
    eta: glam::Vec3,
    k: glam::Vec3,
}

impl ConductorBxdf {
    pub fn new(eta: glam::Vec3, k: glam::Vec3) -> Self {
        Self { eta, k }
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

impl Bxdf for ConductorBxdf {
    fn sample(
        &self,
        _hit_point: glam::Vec3,
        view_direction: glam::Vec3,
        _rng: &mut crate::util::Rng,
    ) -> Option<ScatteringSample> {
        if view_direction.y <= 0.0 {
            return None;
        }

        let light_direction =
            glam::Vec3::new(-view_direction.x, view_direction.y, -view_direction.z);
        let bsdf = fresnel(self.eta, self.k, view_direction.y) / light_direction.y.abs();
        Some(ScatteringSample {
            bsdf,
            pdf: 1.0,
            light_direction,
        })
    }
}
