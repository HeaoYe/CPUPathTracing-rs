use super::bxdf::{Bxdf, ScatteringSample};

pub struct DielectricBxdf {
    ior: f32,
    reflectance: glam::Vec3,
    transmittance: glam::Vec3,
}

impl DielectricBxdf {
    pub fn new(ior: f32, reflectance: glam::Vec3, transmittance: glam::Vec3) -> Self {
        Self {
            ior,
            reflectance,
            transmittance,
        }
    }

    pub fn new_with_tint(ior: f32, tint: glam::Vec3) -> Self {
        Self::new(ior, tint, tint)
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
        let (etai_div_etat, normal, cos_theta_t) = if view_direction.y > 0.0 {
            (self.ior, glam::Vec3::Y, view_direction.y)
        } else {
            (1.0 / self.ior, glam::Vec3::NEG_Y, -view_direction.y)
        };

        let mut cos_theta_i = 0.0;
        let fr = fresnel(etai_div_etat, cos_theta_t, &mut cos_theta_i);

        if rng.uniform() < fr {
            let light_direction =
                glam::Vec3::new(-view_direction.x, view_direction.y, -view_direction.z);
            Some(ScatteringSample {
                bsdf: fr * self.reflectance / light_direction.y.abs(),
                pdf: fr,
                light_direction,
            })
        } else {
            let light_direction = -view_direction / etai_div_etat
                + (cos_theta_t / etai_div_etat - cos_theta_i) * normal;
            Some(ScatteringSample {
                bsdf: (1.0 - fr) * self.transmittance
                    / (etai_div_etat * etai_div_etat)
                    / light_direction.y.abs(),
                pdf: 1.0 - fr,
                light_direction,
            })
        }
    }
}
