use crate::camera::Camera;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backward,
}

pub struct CameraController<'a> {
    pub camera: &'a mut Camera,

    theta: f32,
    phi: f32,
    move_speed: f32,
    turn_speed: glam::Vec2,
}

impl<'a> CameraController<'a> {
    pub fn new(camera: &'a mut Camera) -> Self {
        let theta = camera.model.direction.y.acos().to_degrees();
        let phi = camera
            .model
            .direction
            .z
            .atan2(camera.model.direction.x)
            .to_degrees();
        Self {
            camera,
            theta,
            phi,
            move_speed: 2.0,
            turn_speed: glam::vec2(0.15, 0.07),
        }
    }

    pub fn translate(&mut self, dt: f32, direction: Direction) {
        let mut forward = self.camera.model.direction;
        forward.y = 0.0;
        forward = forward.normalize();

        let move_direction = match direction {
            Direction::Up => glam::Vec3::Y,
            Direction::Down => glam::Vec3::NEG_Y,
            Direction::Left => forward.cross(glam::Vec3::Y),
            Direction::Right => -forward.cross(glam::Vec3::Y),
            Direction::Forward => forward,
            Direction::Backward => -forward,
        };

        self.camera.model.position += dt * self.move_speed * move_direction;
        self.camera.model.caculate_matrix();
    }

    pub fn turn(&mut self, delta: glam::Vec2) {
        self.phi -= delta.x * self.turn_speed.x;
        self.phi = self.phi.rem_euclid(360.0);
        self.theta += delta.y * self.turn_speed.y;
        self.theta = self.theta.clamp(1.0, 179.0);

        let sin_theta = self.theta.to_radians().sin();
        let cos_theta = self.theta.to_radians().cos();
        let sin_phi = self.phi.to_radians().sin();
        let cos_phi = self.phi.to_radians().cos();

        self.camera.model.direction =
            glam::vec3(sin_theta * cos_phi, cos_theta, sin_theta * sin_phi);
        self.camera.model.caculate_matrix();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera.model.vertical_fov = (self.camera.model.vertical_fov - delta).clamp(1.0, 179.0);
        self.camera.model.caculate_matrix();
    }

    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.camera.film.set_resolution(width, height);
        self.camera.model.film_width = width as f32;
        self.camera.model.film_height = height as f32;
    }

    pub fn print(&self) {
        let view_point = self.camera.model.position + self.camera.model.direction;
        println!("Camera:");
        println!(
            "\tFilm Resolution: ({}, {})",
            self.camera.film.width(),
            self.camera.film.height()
        );
        println!("\tPosition: {}", self.camera.model.position);
        println!("\tViewpoint: {}", view_point);
        println!("\tVertical Fov: {}", self.camera.model.vertical_fov);
    }
}
