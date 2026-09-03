use super::Ray;
use crate::{accelerate::Bounds, util::Rng};

pub struct Intersection {
    pub t: f32,
    pub hit_point: glam::Vec3,
    pub normal: glam::Vec3,
}

impl Intersection {
    pub fn new(ray: &Ray, t: f32, normal: glam::Vec3) -> Self {
        let hit_point = ray.at(t);
        Intersection {
            t,
            hit_point,
            normal,
        }
    }
}

pub trait Shape: Sampleable + Sync {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection>;
}

pub trait Bounded {
    fn bounds(&self) -> Bounds;
}

pub trait Centroid {
    fn centroid(&self) -> glam::Vec3;
}

pub struct SurfaceSample {
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub pdf: f32,
}

pub trait Sampleable {
    fn area(&self) -> f32;
    fn sample(&self, rng: &mut Rng) -> Option<SurfaceSample>;
}
