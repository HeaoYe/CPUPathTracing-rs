mod camera;
mod film;
mod frame;
mod material;
mod model;
mod plane;
mod ray;
mod rgb;
mod scene;
mod shape;
mod sphere;
mod spin_lock;
mod thread_pool;
mod triangle;

use rand::RngExt;
use rand_pcg::Pcg32;
use std::sync::atomic::AtomicUsize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_pool = thread_pool::ThreadPool::new(0);

    let width = 192 * 4;
    let height = 108 * 4;
    let film = film::Film::new(width, height);
    let mut camera = camera::Camera::new(
        film,
        glam::Vec3::new(-3.6, 0.0, 0.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        45.0,
    );
    let count = AtomicUsize::new(0);
    let count_ref = &count;

    let model = model::Model::load("models/simple_dragon.obj")?;
    let sphere = sphere::Sphere {
        center: glam::Vec3::ZERO,
        radius: 1.0,
    };
    let plane = plane::Plane {
        point: glam::Vec3::ZERO,
        normal: glam::Vec3::Y,
    };

    let mut scene = scene::Scene::default();
    scene.add_shape(
        &model,
        material::Material::from_lambertian(rgb::RGB::new(202, 159, 117)),
        scene::InstanceTransform {
            scale: glam::Vec3::new(1.0, 3.0, 2.0),
            ..Default::default()
        },
    );
    scene.add_shape(
        &sphere,
        material::Material::from_lambertian_emissive(glam::Vec3::ONE, rgb::RGB::new(255, 128, 128)),
        scene::InstanceTransform {
            translation: glam::Vec3::new(0.0, 0.0, 2.5),
            ..Default::default()
        },
    );
    scene.add_shape(
        &sphere,
        material::Material::from_lambertian_emissive(glam::Vec3::ONE, rgb::RGB::new(128, 128, 255)),
        scene::InstanceTransform {
            translation: glam::Vec3::new(0.0, 0.0, -2.5),
            ..Default::default()
        },
    );
    scene.add_shape(
        &sphere,
        material::Material::from_specular(glam::Vec3::ONE),
        scene::InstanceTransform {
            translation: glam::Vec3::new(3.0, 0.5, -2.0),
            ..Default::default()
        },
    );
    scene.add_shape(
        &plane,
        material::Material::from_lambertian(rgb::RGB::new(120, 204, 157)),
        scene::InstanceTransform {
            translation: glam::Vec3::new(0.0, -0.5, 0.0),
            ..Default::default()
        },
    );

    let spp = 128;

    thread_pool.parallel_for_2d(
        camera.film.width(),
        camera.film.height(),
        camera.film.as_slice_mut(),
        move |x, y, pixel| {
            for i in 0..spp {
                let mut rng = Pcg32::new(0, ((x + 1) * (y + 1) * (i + 1)) as u64);

                let mut ray = camera.geometry.generate_ray(
                    glam::IVec2::new(x as i32, y as i32),
                    glam::Vec2::new(rng.random(), rng.random()),
                );
                let mut beta = glam::Vec3::ONE;
                let mut radiance = glam::Vec3::ZERO;

                let mut depth = 0;

                loop {
                    depth += 1;
                    if depth > 16 {
                        break;
                    }

                    let Some(scene::HitInfo {
                        intersection,
                        material,
                    }) = scene.intersect(&ray, 1e-5, f32::INFINITY)
                    else {
                        break;
                    };

                    radiance += beta * material.emissive;
                    beta *= material.albedo;

                    ray.origin = intersection.hit_point;
                    let frame = frame::Frame::new(intersection.normal);
                    let mut light_direction;
                    if material.is_specular {
                        let view_direction = frame.local_from_world(-ray.direction);
                        light_direction =
                            glam::Vec3::new(-view_direction.x, view_direction.y, -view_direction.z);
                    } else {
                        loop {
                            light_direction = glam::Vec3::new(
                                rng.random_range(-1.0..=1.0),
                                rng.random_range(0.0..=1.0),
                                rng.random_range(-1.0..=1.0),
                            );
                            if light_direction.dot(light_direction) <= 1.0 {
                                break;
                            }
                        }
                        light_direction = light_direction.normalize();
                    }
                    ray.direction = frame.world_from_local(light_direction);
                }

                pixel.add_sample(radiance);
            }

            let progress = count_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if progress.is_multiple_of(width) {
                println!(
                    "Progress: {:.2}%",
                    progress as f32 * 100.0 / ((height - 1) * width) as f32
                );
            }
        },
    );

    camera.film.save("test.ppm")?;

    Ok(())
}
