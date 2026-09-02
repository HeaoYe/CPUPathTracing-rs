mod camera_controller;
mod camera_model;
mod film;

pub use camera_controller::{CameraController, Direction};
pub use camera_model::CameraModel;
pub use film::{Film, PixelSample};

pub struct Camera {
    pub film: Film,
    pub model: CameraModel,
}

impl Camera {
    pub fn new(
        film: Film,
        position: glam::Vec3,
        view_point: glam::Vec3,
        vertical_fov: f32,
    ) -> Self {
        let model = CameraModel::new(
            position,
            (view_point - position).normalize(),
            vertical_fov,
            film.width() as f32,
            film.height() as f32,
        );

        Self { film, model }
    }
}
