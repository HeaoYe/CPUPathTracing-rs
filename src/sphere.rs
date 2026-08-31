use super::ray::Ray;

pub struct Sphere {
    pub center: glam::Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        let co = ray.origin - self.center;
        let b = 2.0 * ray.direction.dot(co);
        let c = co.dot(co) - self.radius * self.radius;
        let delta = b * b - 4.0 * c;
        if delta < 0.0 {
            return None;
        }
        let delta_sqrt = delta.sqrt();
        let mut hit_t = (-b - delta_sqrt) * 0.5;
        if hit_t < 0.0 {
            hit_t = (-b + delta_sqrt) * 0.5;
        }
        if hit_t > 0.0 { Some(hit_t) } else { None }
    }
}
