pub mod uniform {
    pub fn disk(u: f32, v: f32) -> glam::Vec2 {
        let r = u.sqrt();
        let theta = 2.0 * std::f32::consts::PI * v;
        glam::Vec2::new(r * theta.cos(), r * theta.sin())
    }

    pub fn disk_pdf() -> f32 {
        std::f32::consts::FRAC_1_PI
    }

    pub fn hemisphere(u: f32, v: f32) -> glam::Vec3 {
        let y = u;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let phi = 2.0 * std::f32::consts::PI * v;
        glam::vec3(r * phi.cos(), y, r * phi.sin())
    }

    pub fn hemisphere_pdf() -> f32 {
        0.5 * std::f32::consts::FRAC_1_PI
    }

    pub fn sphere(u: f32, v: f32) -> glam::Vec3 {
        let y = 2.0 * u - 1.0;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let phi = 2.0 * std::f32::consts::PI * v;
        glam::vec3(r * phi.cos(), y, r * phi.sin())
    }

    pub fn sphere_pdf() -> f32 {
        0.25 * std::f32::consts::FRAC_1_PI
    }
}

pub mod importance {
    pub fn cosine_hemisphere(u: f32, v: f32) -> glam::Vec3 {
        let r = u.sqrt();
        let phi = 2.0 * std::f32::consts::PI * v;
        glam::vec3(r * phi.cos(), (1.0 - r * r).max(0.0).sqrt(), r * phi.sin())
    }

    pub fn cosine_hemisphere_pdf(direction: glam::Vec3) -> f32 {
        direction.y.max(0.0) / std::f32::consts::PI
    }
}
