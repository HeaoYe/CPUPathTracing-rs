use crate::bsdf::{Bsdf, DiffuseBxdf, SpecularBxdf};

pub struct Material {
    pub bsdf: Bsdf,
    pub emissive: glam::Vec3,
}

impl Default for Material {
    fn default() -> Self {
        Self::diffuse(glam::Vec3::ONE)
    }
}

impl Material {
    fn from_bsdf(bsdf: Bsdf) -> Self {
        Self {
            bsdf,
            emissive: glam::Vec3::ZERO,
        }
    }

    pub fn diffuse(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Diffuse(DiffuseBxdf::new(albedo.into())))
    }

    pub fn specular(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Specular(SpecularBxdf::new(albedo.into())))
    }

    pub fn with_emissive(self, emissive: impl Into<glam::Vec3>) -> Self {
        Self {
            bsdf: self.bsdf,
            emissive: emissive.into(),
        }
    }
}
