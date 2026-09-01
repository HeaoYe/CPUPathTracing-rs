use super::{Bounded, Centroid, Intersection, Ray, Shape};
use crate::accelerate::Bounds;

pub struct Sphere {
    pub center: glam::Vec3,
    pub radius: f32,
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let co = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * ray.direction.dot(co);
        let c = co.dot(co) - self.radius * self.radius;
        let delta = b * b - 4.0 * a * c;
        if delta < 0.0 {
            return None;
        }
        let delta_sqrt = delta.sqrt();
        let mut hit_t = (-b - delta_sqrt) * 0.5 / a;
        if hit_t <= t_min {
            hit_t = (-b + delta_sqrt) * 0.5 / a;
        }
        if hit_t > t_min && hit_t < t_max {
            let hit_point = ray.at(hit_t);
            Some(Intersection::new(
                ray,
                hit_t,
                (hit_point - self.center).normalize(),
            ))
        } else {
            None
        }
    }
}

impl Bounded for Sphere {
    fn bounds(&self) -> Bounds {
        Bounds::new(self.center - self.radius, self.center + self.radius)
    }
}

impl Centroid for Sphere {
    fn centroid(&self) -> glam::Vec3 {
        self.center
    }
}
