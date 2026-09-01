use super::Ray;
use crate::accelerate::Bounds;

#[cfg(debug_assertions)]
#[derive(Default)]
pub struct IntersectionDebugInfo {
    pub bounds_test_count: usize,
    pub triangle_test_count: usize,
    pub bvh_depth: usize,
}

pub struct Intersection {
    pub t: f32,
    pub hit_point: glam::Vec3,
    pub normal: glam::Vec3,

    #[cfg(debug_assertions)]
    pub debug_info: IntersectionDebugInfo,
}

impl Intersection {
    pub fn new(ray: &Ray, t: f32, normal: glam::Vec3) -> Self {
        let hit_point = ray.at(t);
        Intersection {
            t,
            hit_point,
            normal,
            #[cfg(debug_assertions)]
            debug_info: Default::default(),
        }
    }
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
