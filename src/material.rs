use crate::bsdf::{Bsdf, ConductorBxdf, DielectricBxdf, DiffuseBxdf, GroundBxdf, SpecularBxdf};

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

    pub fn conductor(eta: impl Into<glam::Vec3>, k: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Conductor(ConductorBxdf::new(eta.into(), k.into())))
    }

    pub fn dielectric(
        ior: f32,
        reflectance: impl Into<glam::Vec3>,
        transmittance: impl Into<glam::Vec3>,
    ) -> Self {
        Self::from_bsdf(Bsdf::Dielectric(DielectricBxdf::new(
            ior,
            reflectance.into(),
            transmittance.into(),
        )))
    }

    pub fn dielectric_with_tint(ior: f32, tint: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Dielectric(DielectricBxdf::new_with_tint(
            ior,
            tint.into(),
        )))
    }

    pub fn diffuse(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Diffuse(DiffuseBxdf::new(albedo.into())))
    }

    pub fn ground(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Ground(GroundBxdf::new(albedo.into())))
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
