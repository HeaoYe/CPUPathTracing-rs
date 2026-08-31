use crate::geometry::Ray;

pub struct Bounds {
    b_min: glam::Vec3,
    b_max: glam::Vec3,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            b_min: glam::Vec3::INFINITY,
            b_max: glam::Vec3::NEG_INFINITY,
        }
    }
}

impl Bounds {
    pub fn new(b_min: glam::Vec3, b_max: glam::Vec3) -> Self {
        Self { b_min, b_max }
    }

    pub fn expand_point(&mut self, point: glam::Vec3) {
        self.b_min = self.b_min.min(point);
        self.b_max = self.b_max.max(point);
    }

    pub fn expand_bounds(&mut self, bounds: &Self) {
        self.b_min = self.b_min.min(bounds.b_min);
        self.b_max = self.b_max.max(bounds.b_max);
    }

    pub fn has_intersection(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let t1 = (self.b_min - ray.origin) / ray.direction;
        let t2 = (self.b_max - ray.origin) / ray.direction;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let near = tmin.x.max(tmin.y).max(tmin.z).max(t_min);
        let far = tmax.x.min(tmax.y).min(tmax.z).min(t_max);

        near < far
    }
}
