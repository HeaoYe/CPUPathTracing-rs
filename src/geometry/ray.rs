#[cfg(debug_assertions)]
#[derive(Default, Clone, Copy)]
pub struct RayDebugInfo {
    pub bounds_test_count: usize,
    pub primitive_test_count: usize,
}

#[cfg(debug_assertions)]
impl RayDebugInfo {
    pub fn extend(&mut self, ohter: RayDebugInfo) {
        self.bounds_test_count += ohter.bounds_test_count;
        self.primitive_test_count += ohter.primitive_test_count
    }
}

pub struct Ray {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,

    #[cfg(debug_assertions)]
    pub debug_info: std::cell::RefCell<RayDebugInfo>,
}

impl Ray {
    pub fn new(origin: glam::Vec3, direction: glam::Vec3) -> Self {
        Self {
            origin,
            direction,
            #[cfg(debug_assertions)]
            debug_info: Default::default(),
        }
    }

    pub fn at(&self, t: f32) -> glam::Vec3 {
        self.origin + t * self.direction
    }

    pub fn transform(&self, matrix: glam::Affine3A) -> Self {
        Self {
            origin: matrix.transform_point3(self.origin),
            direction: matrix.transform_vector3(self.direction),

            #[cfg(debug_assertions)]
            debug_info: Default::default(),
        }
    }
}
