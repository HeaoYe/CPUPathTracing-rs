use crate::{
    bsdf::{Bsdf, ConductorBsdf, DielectricBsdf, DiffuseBsdf, GroundBsdf, SpecularBsdf},
    spectrum::Spectrum,
};

#[derive(Clone, Copy)]
pub struct Material<'a> {
    pub bsdf: Bsdf<'a>,
}

impl Default for Material<'_> {
    fn default() -> Self {
        Self::diffuse(&Spectrum::Default)
    }
}

impl<'a> Material<'a> {
    fn from_bsdf(bsdf: Bsdf<'a>) -> Self {
        Self { bsdf }
    }

    pub fn conductor_with_alpha(
        eta: &'a Spectrum<'a>,
        k: &'a Spectrum<'a>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self::from_bsdf(Bsdf::Conductor(ConductorBsdf::new(
            eta, k, alpha_x, alpha_z,
        )))
    }

    pub fn conductor(eta: &'a Spectrum<'a>, k: &'a Spectrum<'a>) -> Self {
        Self::conductor_with_alpha(eta, k, 0.0, 0.0)
    }

    pub fn dielectric_with_alpha(
        ior: &'a Spectrum<'a>,
        reflectance: &'a Spectrum<'a>,
        transmittance: &'a Spectrum<'a>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self::from_bsdf(Bsdf::Dielectric(DielectricBsdf::new(
            ior,
            reflectance,
            transmittance,
            alpha_x,
            alpha_z,
        )))
    }

    pub fn dielectric(
        ior: &'a Spectrum<'a>,
        reflectance: &'a Spectrum<'a>,
        transmittance: &'a Spectrum<'a>,
    ) -> Self {
        Self::dielectric_with_alpha(ior, reflectance, transmittance, 0.0, 0.0)
    }

    pub fn dielectric_with_alpha_tint(
        ior: &'a Spectrum<'a>,
        tint: &'a Spectrum<'a>,
        alpha_x: f32,
        alpha_z: f32,
    ) -> Self {
        Self::dielectric_with_alpha(ior, tint, tint, alpha_x, alpha_z)
    }

    pub fn dielectric_with_tint(ior: &'a Spectrum<'a>, tint: &'a Spectrum<'a>) -> Self {
        Self::dielectric_with_alpha_tint(ior, tint, 0.0, 0.0)
    }

    pub fn diffuse(albedo: &'a Spectrum<'a>) -> Self {
        Self::from_bsdf(Bsdf::Diffuse(DiffuseBsdf::new(albedo)))
    }

    pub fn ground(albedo: &'a Spectrum<'a>) -> Self {
        Self::from_bsdf(Bsdf::Ground(GroundBsdf::new(albedo)))
    }

    pub fn specular(albedo: &'a Spectrum<'a>) -> Self {
        Self::from_bsdf(Bsdf::Specular(SpecularBsdf::new(albedo)))
    }
}
