use cpu_path_tracing::{camera, geometry, integrator, material, scene, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 4, 108 * 4);
    let mut camera = camera::Camera::new(
        film,
        glam::Vec3::new(-3.6, 0.0, 0.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        45.0,
    );

    let model = geometry::Model::load("models/dragon_87k.obj")?;
    // let sphere = geometry::Sphere {
    //     center: glam::Vec3::ZERO,
    //     radius: 1.0,
    // };
    // let plane = geometry::Plane {
    //     point: glam::Vec3::ZERO,
    //     normal: glam::Vec3::Y,
    // };

    let mut scene = scene::Scene::default();
    scene.add_shape(
        &model,
        material::Material::from_lambertian(util::Rgb::new(202, 159, 117)),
        scene::InstanceTransform {
            scale: glam::Vec3::new(3.0, 3.0, 3.0),
            ..Default::default()
        },
    );
    // scene.add_shape(
    //     &sphere,
    //     material::Material::from_lambertian_emissive(
    //         glam::Vec3::ONE,
    //         util::Rgb::new(255, 128, 128),
    //     ),
    //     scene::InstanceTransform {
    //         translation: glam::Vec3::new(0.0, 0.0, 2.5),
    //         ..Default::default()
    //     },
    // );
    // scene.add_shape(
    //     &sphere,
    //     material::Material::from_lambertian_emissive(
    //         glam::Vec3::ONE,
    //         util::Rgb::new(128, 128, 255),
    //     ),
    //     scene::InstanceTransform {
    //         translation: glam::Vec3::new(0.0, 0.0, -2.5),
    //         ..Default::default()
    //     },
    // );
    // scene.add_shape(
    //     &sphere,
    //     material::Material::from_specular(glam::Vec3::ONE),
    //     scene::InstanceTransform {
    //         translation: glam::Vec3::new(3.0, 0.5, -2.0),
    //         ..Default::default()
    //     },
    // );
    // scene.add_shape(
    //     &plane,
    //     material::Material::from_lambertian(util::Rgb::new(120, 204, 157)),
    //     scene::InstanceTransform {
    //         translation: glam::Vec3::new(0.0, -0.5, 0.0),
    //         ..Default::default()
    //     },
    // );

    integrator::render(
        &integrator::NormalIntegrator,
        &mut camera,
        &scene,
        1,
        "normal.ppm",
    )?;

    integrator::render(
        &integrator::BoundsTestIntegrator,
        &mut camera,
        &scene,
        1,
        "bounds_test.ppm",
    )?;

    integrator::render(
        &integrator::TriangleTestIntegrator,
        &mut camera,
        &scene,
        1,
        "triangle_test.ppm",
    )?;

    integrator::render(
        &integrator::BvhDepthIntegrator,
        &mut camera,
        &scene,
        1,
        "bvh_depth.ppm",
    )?;

    integrator::render(
        &integrator::SimpleRTIntegrator,
        &mut camera,
        &scene,
        128,
        "test.ppm",
    )?;

    Ok(())
}
