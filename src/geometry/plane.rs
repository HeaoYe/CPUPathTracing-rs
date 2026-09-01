use super::{Intersection, Ray, Shape};

pub struct Plane {
    pub point: glam::Vec3,
    pub normal: glam::Vec3,
}

impl Shape for Plane {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let hit_t = self.normal.dot(self.point - ray.origin) / self.normal.dot(ray.direction);
        if hit_t > t_min && hit_t < t_max {
            Some(Intersection::new(ray, hit_t, self.normal))
        } else {
            None
        }
    }
}
