use super::Ray;
use crate::accelerate::Bounds;

pub struct Intersection {
    pub t: f32,
    pub hit_point: glam::Vec3,
    pub normal: glam::Vec3,
}

pub trait Shape: Sync {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection>;
}

pub trait Bounded {
    fn bounds(&self) -> Bounds;
}

pub trait Centroid {
    fn centroid(&self) -> glam::Vec3;
}
