use cpu_path_tracing::{camera, geometry, integrator, material, scene, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 4, 108 * 4);
    let mut camera =
        camera::Camera::new(film, [-10.0, 1.5, 0.0].into(), [0.0, 0.0, 0.0].into(), 45.0);

    let model = geometry::Model::load("models/dragon_871k.obj")?;
    let sphere = geometry::Sphere {
        center: glam::Vec3::ZERO,
        radius: 1.0,
    };
    let plane = geometry::Plane::new(glam::Vec3::ZERO, glam::Vec3::Y, f32::MAX);

    let mut builder = scene::SceneBuilder::default();

    for i in -3..=3 {
        builder.add_shape(
            &sphere,
            material::Material::dielectric_with_tint(1.0 + 0.2 * (i + 3) as f32, glam::Vec3::ONE),
            scene::InstanceTransform {
                translation: [0.0, 0.5, i as f32 * 2.0].into(),
                scale: glam::Vec3::splat(0.8),
                ..Default::default()
            },
        );
    }

    for i in -3..=3 {
        let c = glam::Vec3::from(util::Rgb::generate_heatmap_rgb((i as f32 + 3.0) / 6.0));
        builder.add_shape(
            &sphere,
            material::Material::conductor(2.0 - c * 2.0, 2.0 + c * 3.0),
            scene::InstanceTransform {
                translation: [0.0, 2.5, i as f32 * 2.0].into(),
                scale: glam::Vec3::splat(0.8),
                ..Default::default()
            },
        );
    }

    builder.add_shape(
        &model,
        material::Material::dielectric_with_tint(1.8, util::Rgb::new(128, 191, 131)),
        scene::InstanceTransform {
            translation: [-5.0, 0.4, 1.5].into(),
            scale: glam::Vec3::splat(2.0),
            ..Default::default()
        },
    );

    builder.add_shape(
        &model,
        material::Material::conductor([0.1, 1.2, 1.8], [5.0, 2.5, 2.0]),
        scene::InstanceTransform {
            translation: [-5.0, 0.4, -1.5].into(),
            scale: glam::Vec3::splat(2.0),
            ..Default::default()
        },
    );

    builder.add_shape(
        &plane,
        material::Material::ground(util::Rgb::new(120, 204, 157)),
        scene::InstanceTransform {
            translation: [0.0, -0.5, 0.0].into(),
            ..Default::default()
        },
    );

    builder.add_shape(
        &plane,
        material::Material::default().with_emissive([0.75, 0.75, 0.8]),
        scene::InstanceTransform {
            translation: [0.0, 10.0, 0.0].into(),
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
        &integrator::SimplePathTracingIntegrator,
        &mut camera,
        &scene,
        128,
        "pt_cosine_test.ppm",
    )?;

    Ok(())
}
