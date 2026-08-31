mod camera;
mod film;
mod ray;
mod sphere;
mod spin_lock;
mod thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_pool = thread_pool::ThreadPool::new(0);

    let film = film::Film::new(1920, 1080);
    let mut camera = camera::Camera::new(
        film,
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        90.0,
    );

    let sphere = sphere::Sphere {
        center: glam::Vec3::ZERO,
        radius: 0.5,
    };
    let light_pos = glam::Vec3::new(1.0, 1.0, 1.0);

    thread_pool.parallel_for_2d(
        camera.film.width(),
        camera.film.height(),
        camera.film.as_slice_mut(),
        move |x, y, pixel| {
            let ray = camera
                .geometry
                .generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(hit_t) = sphere.intersect(&ray) {
                let hit_point = ray.at(hit_t);
                let normal = (hit_point - sphere.center).normalize();
                let light_direction = (light_pos - hit_point).normalize();
                let cosine = normal.dot(light_direction).max(0.0);
                *pixel = glam::Vec3::splat(cosine);
            }
        },
    );

    camera.film.save("test.ppm")?;

    Ok(())
}
