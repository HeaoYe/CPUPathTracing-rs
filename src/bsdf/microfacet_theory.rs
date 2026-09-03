use crate::{geometry::Frame, sample::uniform, util::Rng};

pub(super) struct MicrofacetTheory {
    alpha_x: f32,
    alpha_z: f32,
}

impl MicrofacetTheory {
    pub(super) fn new(alpha_x: f32, alpha_z: f32) -> Self {
        Self {
            alpha_x: (alpha_x * alpha_x).clamp(1e-3, 1.0),
            alpha_z: (alpha_z * alpha_z).clamp(1e-3, 1.0),
        }
    }

    pub(super) fn normal_distribution(&self, microfacet_normal: glam::Vec3) -> f32 {
        let mut slope = glam::Vec2::new(
            -microfacet_normal.x / microfacet_normal.y,
            -microfacet_normal.z / microfacet_normal.y,
        );
        slope.x /= self.alpha_x;
        slope.y /= self.alpha_z;
        let slope_distribution = self.slope_distribution(slope) / (self.alpha_x * self.alpha_z);
        slope_distribution / microfacet_normal.y.powi(4)
    }

    pub(super) fn masking(&self, view_direction: glam::Vec3, microfacet_normal: glam::Vec3) -> f32 {
        let view_direction_upper = if view_direction.y > 0.0 {
            view_direction
        } else {
            -view_direction
        };

        if view_direction_upper.dot(microfacet_normal) <= 0.0 {
            0.0
        } else {
            1.0 / (1.0 + self.lambda(view_direction_upper))
        }
    }

    pub(super) fn height_correlated_masking_shadowing(
        &self,
        light_direction: glam::Vec3,
        view_direction: glam::Vec3,
        microfacet_normal: glam::Vec3,
    ) -> f32 {
        let light_direction_upper = if light_direction.y > 0.0 {
            light_direction
        } else {
            -light_direction
        };
        if light_direction_upper.dot(microfacet_normal) <= 0.0 {
            return 0.0;
        }

        let view_direction_upper = if view_direction.y > 0.0 {
            view_direction
        } else {
            -view_direction
        };
        if view_direction_upper.dot(microfacet_normal) <= 0.0 {
            return 0.0;
        }

        1.0 / (1.0 + self.lambda(light_direction_upper) + self.lambda(view_direction_upper))
    }

    pub(super) fn visible_normal_distribution(
        &self,
        view_direction: glam::Vec3,
        microfacet_normal: glam::Vec3,
    ) -> f32 {
        let view_direction_upper = if view_direction.y > 0.0 {
            view_direction
        } else {
            -view_direction
        };

        let cos_theta_o = view_direction_upper.dot(microfacet_normal);
        if cos_theta_o <= 0.0 {
            0.0
        } else {
            self.normal_distribution(microfacet_normal)
                * cos_theta_o
                * self.masking(view_direction, microfacet_normal)
                / view_direction_upper.y
        }
    }

    pub(super) fn sample_visible_normal(
        &self,
        view_direction: glam::Vec3,
        rng: &mut Rng,
    ) -> glam::Vec3 {
        let view_direction_upper = if view_direction.y > 0.0 {
            view_direction
        } else {
            -view_direction
        };

        let view_direction_hemi = glam::vec3(
            self.alpha_x * view_direction_upper.x,
            view_direction_upper.y,
            self.alpha_z * view_direction_upper.z,
        )
        .normalize();

        let mut sample = uniform::disk(rng.uniform(), rng.uniform());
        let h = (1.0 - sample.x * sample.x).max(0.0).sqrt();
        let t = 0.5 * (1.0 + view_direction_hemi.y);
        sample.y = t * sample.y + (1.0 - t) * h;

        let frame = Frame::new(view_direction_hemi);
        let microfacet_normal_hemi = frame.world_from_local(glam::vec3(
            sample.x,
            (1.0 - sample.x * sample.x - sample.y * sample.y)
                .max(0.0)
                .sqrt(),
            sample.y,
        ));

        glam::vec3(
            self.alpha_x * microfacet_normal_hemi.x,
            microfacet_normal_hemi.y,
            self.alpha_z * microfacet_normal_hemi.z,
        )
        .normalize()
    }

    pub(super) fn is_delta_distribution(&self) -> bool {
        self.alpha_x.max(self.alpha_z) == 1e-3
    }

    fn slope_distribution(&self, slope: glam::Vec2) -> f32 {
        1.0 / (std::f32::consts::PI * (1.0 + slope.x * slope.x + slope.y * slope.y).powi(2))
    }
    fn lambda(&self, direction_upper: glam::Vec3) -> f32 {
        if direction_upper.y == 0.0 {
            return f32::INFINITY;
        }

        let squared = direction_upper * direction_upper;
        let length_squared = squared.x + squared.y + squared.z;
        if length_squared == 0.0 {
            return 0.0;
        }

        let cos2_phi = squared.x / length_squared;
        let sin2_phi = squared.z / length_squared;
        let tan2_theta = length_squared / squared.y;
        let alpha0_2 =
            self.alpha_x * self.alpha_x * cos2_phi + self.alpha_z * self.alpha_z * sin2_phi;
        0.5 * ((1.0 + alpha0_2 * tan2_theta).sqrt() - 1.0)
    }
}
