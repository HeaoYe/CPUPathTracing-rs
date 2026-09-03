use crate::bsdf::{Bsdf, ConductorBsdf, DielectricBsdf, DiffuseBsdf, GroundBsdf, SpecularBsdf};

pub struct Material {
    pub bsdf: Bsdf,
}

impl Default for Material {
    fn default() -> Self {
        Self::diffuse(glam::Vec3::ONE)
    }
}

impl Material {
    fn from_bsdf(bsdf: Bsdf) -> Self {
        Self { bsdf }
    }

    pub fn conductor_with_alpha(
        eta: impl Into<glam::Vec3>,
        k: impl Into<glam::Vec3>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self::from_bsdf(Bsdf::Conductor(ConductorBsdf::new(
            eta.into(),
            k.into(),
            alpha_x,
            alpha_z,
        )))
    }

    pub fn conductor(eta: impl Into<glam::Vec3>, k: impl Into<glam::Vec3>) -> Self {
        Self::conductor_with_alpha(eta, k, 0.0, 0.0)
    }

    pub fn dielectric_with_alpha(
        ior: f32,
        reflectance: impl Into<glam::Vec3>,
        transmittance: impl Into<glam::Vec3>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self::from_bsdf(Bsdf::Dielectric(DielectricBsdf::new(
            ior,
            reflectance.into(),
            transmittance.into(),
            alpha_x,
            alpha_z,
        )))
    }

    pub fn dielectric(
        ior: f32,
        reflectance: impl Into<glam::Vec3>,
        transmittance: impl Into<glam::Vec3>,
    ) -> Self {
        Self::dielectric_with_alpha(ior, reflectance, transmittance, 0.0, 0.0)
    }

    pub fn dielectric_with_alpha_tint(
        ior: f32,
        tint: impl Into<glam::Vec3>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        let tint = tint.into();
        Self::dielectric_with_alpha(ior, tint, tint, alpha_x, alpha_z)
    }

    pub fn dielectric_with_tint(ior: f32, tint: impl Into<glam::Vec3>) -> Self {
        Self::dielectric_with_alpha_tint(ior, tint, 0.0, 0.0)
    }

    pub fn diffuse(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Diffuse(DiffuseBsdf::new(albedo.into())))
    }

    pub fn ground(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Ground(GroundBsdf::new(albedo.into())))
    }

    pub fn specular(albedo: impl Into<glam::Vec3>) -> Self {
        Self::from_bsdf(Bsdf::Specular(SpecularBsdf::new(albedo.into())))
    }
}
