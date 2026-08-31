use super::film::Film;
use super::ray::Ray;

pub struct CameraGeometry {
    position: glam::Vec3,
    direction: glam::Vec3,
    vertical_fov: f32,
    film_width: f32,
    film_height: f32,

    camera_from_clip: glam::Mat4,
    world_from_camera: glam::Affine3A,
}

impl CameraGeometry {
    fn new(
        position: glam::Vec3,
        direction: glam::Vec3,
        vertical_fov: f32,
        film_width: f32,
        film_height: f32,
    ) -> Self {
        let mut geometry = Self {
            position,
            direction,
            vertical_fov,
            film_width,
            film_height,
            camera_from_clip: glam::Mat4::ZERO,
            world_from_camera: glam::Affine3A::ZERO,
        };
        geometry.caculate_matrix();
        geometry
    }

    fn caculate_matrix(&mut self) {
        let clip_from_camera = glam::camera::lh::proj::directx::perspective(
            self.vertical_fov.to_radians(),
            self.film_width / self.film_height,
            1.0,
            2.0,
        );
        self.camera_from_clip = clip_from_camera.inverse();

        let camera_from_world = glam::camera::lh::view::look_at_affine3a(
            self.position,
            self.position + self.direction,
            glam::Vec3::Y,
        );
        self.world_from_camera = camera_from_world.inverse();
    }

    pub fn generate_ray(&self, pixel_coord: glam::IVec2, offset: glam::Vec2) -> Ray {
        let mut ndc =
            (pixel_coord.as_vec2() + offset) / glam::Vec2::new(self.film_width, self.film_height);
        ndc.y = 1.0 - ndc.y;
        ndc = 2.0 * ndc - 1.0;
        let clip = ndc.extend(0.0).extend(1.0);
        let world = (self.world_from_camera * self.camera_from_clip * clip).truncate();
        Ray {
            origin: self.position,
            direction: (world - self.position).normalize(),
        }
    }
}

pub struct Camera {
    pub film: Film,
    pub geometry: CameraGeometry,
}

impl Camera {
    pub fn new(
        film: Film,
        position: glam::Vec3,
        view_point: glam::Vec3,
        vertical_fov: f32,
    ) -> Self {
        let geometry = CameraGeometry::new(
            position,
            (view_point - position).normalize(),
            vertical_fov,
            film.width() as f32,
            film.height() as f32,
        );

        Self { film, geometry }
    }
}
