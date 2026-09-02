pub struct Frame {
    x_axis: glam::Vec3,
    y_axis: glam::Vec3,
    z_axis: glam::Vec3,
}

impl Frame {
    pub fn new(normal: glam::Vec3) -> Self {
        let y_axis = normal.normalize();
        let up = if y_axis.y.abs() < 0.99999 {
            glam::Vec3::Y
        } else {
            glam::Vec3::Z
        };
        let x_axis = y_axis.cross(up).normalize();
        let z_axis = x_axis.cross(y_axis).normalize();

        Self {
            x_axis,
            y_axis,
            z_axis,
        }
    }

    pub fn world_from_local(&self, direction_local: glam::Vec3) -> glam::Vec3 {
        direction_local.x * self.x_axis
            + direction_local.y * self.y_axis
            + direction_local.z * self.z_axis
    }

    pub fn local_from_world(&self, direction_world: glam::Vec3) -> glam::Vec3 {
        glam::vec3(
            direction_world.dot(self.x_axis),
            direction_world.dot(self.y_axis),
            direction_world.dot(self.z_axis),
        )
    }
}
