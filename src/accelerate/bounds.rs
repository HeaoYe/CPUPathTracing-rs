use crate::geometry::Ray;

#[derive(Clone, Copy)]
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

    pub fn extend_point(&mut self, point: glam::Vec3) {
        self.b_min = self.b_min.min(point);
        self.b_max = self.b_max.max(point);
    }

    pub fn extend_bounds(&mut self, bounds: Self) {
        self.b_min = self.b_min.min(bounds.b_min);
        self.b_max = self.b_max.max(bounds.b_max);
    }

    pub fn has_intersection(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let t1 = (self.b_min - ray.origin) / ray.direction;
        let t2 = (self.b_max - ray.origin) / ray.direction;

        let near = t1.min(t2).max_element().max(t_min);
        let far = t1.max(t2).min_element().min(t_max);

        near < far
    }

    pub fn has_intersection_inv_dir(
        &self,
        origin: glam::Vec3,
        inv_direction: glam::Vec3,
        t_min: f32,
        t_max: f32,
    ) -> bool {
        let t1 = (self.b_min - origin) * inv_direction;
        let t2 = (self.b_max - origin) * inv_direction;

        let near = t1.min(t2).max_element().max(t_min);
        let far = t1.max(t2).min_element().min(t_max);

        near < far
    }

    pub fn diagonal(&self) -> glam::Vec3 {
        self.b_max - self.b_min
    }

    pub fn area(&self) -> f32 {
        let diag = self.diagonal();
        (diag.x * (diag.y + diag.z) + diag.y * diag.z) * 2.0
    }

    pub fn corner(&self, idx: usize) -> glam::Vec3 {
        let mut corner = self.b_max;
        if (idx & 0b1) == 0 {
            corner.x = self.b_min.x;
        }
        if (idx & 0b10) == 0 {
            corner.y = self.b_min.y;
        }
        if (idx & 0b100) == 0 {
            corner.z = self.b_min.z;
        }
        corner
    }

    pub fn b_min(&self) -> glam::Vec3 {
        self.b_min
    }

    pub fn b_max(&self) -> glam::Vec3 {
        self.b_max
    }
}
