mod camera;
mod film;
mod model;
mod ray;
mod shape;
mod sphere;
mod spin_lock;
mod thread_pool;
mod triangle;

use std::sync::atomic::AtomicI32;

use shape::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_pool = thread_pool::ThreadPool::new(0);

    let film = film::Film::new(1920, 1080);
    let mut camera = camera::Camera::new(
        film,
        glam::Vec3::new(-0.6, 0.0, 0.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        90.0,
    );
    let count = AtomicI32::new(0);
    let count_ref = &count;

    let model = model::Model::load("models/simple_dragon.obj")?;
    let light_pos = glam::Vec3::new(-1.0, 2.0, 1.0);

    thread_pool.parallel_for_2d(
        camera.film.width(),
        camera.film.height(),
        camera.film.as_slice_mut(),
        move |x, y, pixel| {
            let ray = camera
                .geometry
                .generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(intersection) = model.intersect(&ray, 1e-3, f32::INFINITY) {
                let light_direction = (light_pos - intersection.hit_point).normalize();
                let cosine = intersection.normal.dot(light_direction).max(0.0);
                *pixel = glam::Vec3::splat(cosine);
            }
            let progress = count_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if progress % 1080 == 0 {
                println!(
                    "Progress: {:.2}%",
                    progress as f32 * 100.0 / ((1920.0 - 1.0) * 1080.0)
                );
            }
        },
    );

    camera.film.save("test.ppm")?;

    Ok(())
}
