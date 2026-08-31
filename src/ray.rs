pub struct Ray {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> glam::Vec3 {
        self.origin + t * self.direction
    }

    pub fn transform(&self, matrix: glam::Affine3A) -> Self {
        Self {
            origin: matrix.transform_point3(self.origin),
            direction: matrix.transform_vector3(self.direction),
        }
    }
}
