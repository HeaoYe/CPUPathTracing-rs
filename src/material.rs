pub struct Material {
    pub albedo: glam::Vec3,
    pub is_specular: bool,
    pub emissive: glam::Vec3,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: glam::Vec3::splat(1.0),
            is_specular: false,
            emissive: glam::Vec3::ZERO,
        }
    }
}

impl Material {
    pub fn from_lambertian(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_lambertian_emissive(albedo.into(), glam::Vec3::ZERO)
    }

    pub fn from_specular(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_specular_emissive(albedo.into(), glam::Vec3::ZERO)
    }

    pub fn from_lambertian_emissive(
        albedo: impl Into<glam::Vec3>,
        emissive: impl Into<glam::Vec3>,
    ) -> Self {
        Self {
            albedo: albedo.into(),
            is_specular: false,
            emissive: emissive.into(),
        }
    }

    pub fn from_specular_emissive(
        albedo: impl Into<glam::Vec3>,
        emissive: impl Into<glam::Vec3>,
    ) -> Self {
        Self {
            albedo: albedo.into(),
            is_specular: true,
            emissive: emissive.into(),
        }
    }
}
