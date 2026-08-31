mod camera;
mod film;
mod model;
mod plane;
mod ray;
mod scene;
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
        glam::Vec3::new(-1.6, 0.0, 0.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        90.0,
    );
    let count = AtomicI32::new(0);
    let count_ref = &count;

    let model = model::Model::load("models/simple_dragon.obj")?;
    let sphere = sphere::Sphere {
        center: glam::Vec3::ZERO,
        radius: 0.5,
    };
    let plane = plane::Plane {
        point: glam::Vec3::ZERO,
        normal: glam::Vec3::Y,
    };

    let mut scene = scene::Scene::default();
    scene.add_shape(
        &model,
        scene::InstanceTransform {
            scale: glam::Vec3::new(1.0, 3.0, 2.0),
            ..Default::default()
        },
    );
    scene.add_shape(
        &sphere,
        scene::InstanceTransform {
            translation: glam::Vec3::new(0.0, 0.0, 1.5),
            scale: glam::Vec3::splat(0.3),
            ..Default::default()
        },
    );
    scene.add_shape(
        &plane,
        scene::InstanceTransform {
            translation: glam::Vec3::new(1.0, -0.5, 0.0),
            ..Default::default()
        },
    );
    let light_pos = glam::Vec3::new(-1.0, 2.0, 1.0);

    thread_pool.parallel_for_2d(
        camera.film.width(),
        camera.film.height(),
        camera.film.as_slice_mut(),
        move |x, y, pixel| {
            let ray = camera
                .geometry
                .generate_ray(glam::IVec2::new(x as i32, y as i32), glam::Vec2::splat(0.5));
            if let Some(intersection) = scene.intersect(&ray, 1e-3, f32::INFINITY) {
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
