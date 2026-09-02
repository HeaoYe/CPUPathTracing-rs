use cpu_path_tracing::{camera, geometry, integrator, material, scene, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 4, 108 * 4);
    let mut camera = camera::Camera::new(
        film,
        glam::Vec3::new(-12.0, 5.0, -12.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        45.0,
    );

    let model = geometry::Model::load("models/dragon_871k.obj")?;
    let sphere = geometry::Sphere {
        center: glam::Vec3::ZERO,
        radius: 1.0,
    };
    let plane = geometry::Plane::new(glam::Vec3::ZERO, glam::Vec3::Y, 100.0);

    let mut builder = scene::SceneBuilder::default();
    let mut rng = util::Rng::new(1234, 0);
    for _ in 0..10000 {
        let mut random_pos = glam::Vec3::new(
            rng.uniform() * 100.0 - 50.0,
            rng.uniform() * 2.0,
            rng.uniform() * 100.0 - 50.0,
        );

        let u = rng.uniform();
        if u < 0.9 {
            builder.add_shape(
                &model,
                material::Material {
                    albedo: util::Rgb::new(202, 159, 117).into(),
                    is_specular: rng.uniform() < 0.5,
                    emissive: glam::Vec3::ZERO,
                },
                scene::InstanceTransform {
                    translation: random_pos,
                    rotation: glam::Quat::from_rotation_x((rng.uniform() * 360.0).to_radians())
                        * glam::Quat::from_rotation_y((rng.uniform() * 360.0).to_radians())
                        * glam::Quat::from_rotation_z((rng.uniform() * 360.0).to_radians()),
                    ..Default::default()
                },
            );
        } else if u < 0.95 {
            builder.add_shape(
                &sphere,
                material::Material::from_specular(glam::Vec3::new(
                    rng.uniform(),
                    rng.uniform(),
                    rng.uniform(),
                )),
                scene::InstanceTransform {
                    translation: random_pos,
                    scale: glam::Vec3::splat(0.3),
                    ..Default::default()
                },
            );
        } else {
            random_pos.y += 6.0;
            builder.add_shape(
                &sphere,
                material::Material::from_lambertian_emissive(
                    glam::Vec3::ONE,
                    glam::Vec3::new(
                        rng.uniform() * 4.0,
                        rng.uniform() * 4.0,
                        rng.uniform() * 4.0,
                    ),
                ),
                scene::InstanceTransform {
                    translation: random_pos,
                    ..Default::default()
                },
            );
        }
    }

    builder.add_shape(
        &plane,
        material::Material::from_lambertian(util::Rgb::new(120, 204, 157)),
        scene::InstanceTransform {
            translation: glam::Vec3::new(0.0, -0.5, 0.0),
            ..Default::default()
        },
    );

    let scene = builder.build();

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
        &integrator::PrimitiveTestIntegrator,
        &mut camera,
        &scene,
        1,
        "primitive_test.ppm",
    )?;

    integrator::render(
        &integrator::SimpleRtIntegrator,
        &mut camera,
        &scene,
        128,
        "rt_test.ppm",
    )?;

    integrator::render(
        &integrator::SimplePathTracingIntegrator,
        &mut camera,
        &scene,
        128,
        "pt_cosine_test.ppm",
    )?;

    Ok(())
}
