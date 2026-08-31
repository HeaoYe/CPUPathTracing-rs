use super::{Intersection, Ray, Shape};

pub struct Triangle {
    pub p0: glam::Vec3,
    pub p1: glam::Vec3,
    pub p2: glam::Vec3,
    pub n0: glam::Vec3,
    pub n1: glam::Vec3,
    pub n2: glam::Vec3,
}

impl Triangle {
    pub fn from_points(p0: glam::Vec3, p1: glam::Vec3, p2: glam::Vec3) -> Self {
        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let normal = e1.cross(e2).normalize();
        Self {
            p0,
            p1,
            p2,
            n0: normal,
            n1: normal,
            n2: normal,
        }
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let e1 = self.p1 - self.p0;
        let e2 = self.p2 - self.p0;
        let s1 = ray.direction.cross(e2);
        let inv_det = 1.0 / s1.dot(e1);

        let s = ray.origin - self.p0;
        let u = s1.dot(s) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s2 = s.cross(e1);
        let v = s2.dot(ray.direction) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let hit_t = s2.dot(e2) * inv_det;
        if hit_t > t_min && hit_t < t_max {
            let hit_point = ray.at(hit_t);
            let normal = ((1.0 - u - v) * self.n0 + u * self.n1 + v * self.n2).normalize();
            Some(Intersection {
                t: hit_t,
                hit_point,
                normal,
            })
        } else {
            None
        }
    }
}
