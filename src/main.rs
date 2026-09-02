use cpu_path_tracing::{camera, geometry, integrator, material, scene, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 10, 108 * 10);
    let mut camera = camera::Camera::new(
        film,
        glam::vec3(-3.488137, 0.184000, -2.268835),
        glam::vec3(-4.255267, 0.356399, -1.650943),
        68.0,
    );

    let model = geometry::Model::load("models/dragon_871k.obj")?;
    let sphere = geometry::Sphere {
        center: glam::Vec3::ZERO,
        radius: 1.0,
    };
    let plane = geometry::Plane::new(glam::Vec3::ZERO, glam::Vec3::Y, 100.0);

    let mut builder = scene::SceneBuilder::default();

    for i in -3..=3 {
        builder.add_shape(
            &sphere,
            material::Material::dielectric_with_alpha_tint(
                1.0 + 0.2 * (i + 3) as f32,
                glam::Vec3::ONE,
                (3.0 - i as f32) / 18.0,
                (3.0 - i as f32) / 6.0,
            ),
            scene::InstanceTransform {
                translation: glam::vec3(0.0, 0.5, i as f32 * 2.0),
                scale: glam::Vec3::splat(0.8),
                ..Default::default()
            },
        );
    }

    for i in -3..=3 {
        let c = glam::Vec3::from(util::Rgb::generate_heatmap_rgb((i as f32 + 3.0) / 6.0));
        builder.add_shape(
            &sphere,
            material::Material::conductor_with_alpha(
                2.0 - c * 2.0,
                2.0 + c * 3.0,
                (3.0 - i as f32) / 6.0,
                (3.0 - i as f32) / 18.0,
            ),
            scene::InstanceTransform {
                translation: glam::vec3(0.0, 2.5, i as f32 * 2.0),
                scale: glam::Vec3::splat(0.8),
                ..Default::default()
            },
        );
    }

    builder.add_shape(
        &model,
        material::Material::dielectric_with_alpha_tint(
            1.8,
            util::Rgb::new(128, 211, 131),
            0.4,
            0.4,
        ),
        scene::InstanceTransform {
            translation: glam::vec3(-5.0, 0.4, 1.5),
            scale: glam::Vec3::splat(2.0),
            ..Default::default()
        },
    );

    builder.add_shape(
        &model,
        material::Material::conductor_with_alpha([0.1, 1.2, 1.8], [5.0, 2.5, 2.0], 0.4, 0.4),
        scene::InstanceTransform {
            translation: glam::vec3(-5.0, 0.4, -1.5),
            scale: glam::Vec3::splat(2.0),
            ..Default::default()
        },
    );

    builder.add_shape(
        &plane,
        material::Material::ground(util::Rgb::new(120, 204, 157)),
        scene::InstanceTransform {
            translation: glam::vec3(0.0, -0.5, 0.0),
            ..Default::default()
        },
    );

    builder.add_shape(
        &sphere,
        material::Material::default().with_emissive([0.95 * 5.0, 0.95 * 5.0, 1.0 * 5.0]),
        scene::InstanceTransform {
            translation: glam::vec3(-2.0, 6.0, 0.0),
            scale: glam::Vec3::splat(2.0),
            ..Default::default()
        },
    );

    let scene = builder.build();

    let simple_path_tracing_integrator = integrator::SimplePathTracingIntegrator;
    if integrator::preview(&simple_path_tracing_integrator, &mut camera, &scene) {
        integrator::render(
            &simple_path_tracing_integrator,
            &mut camera,
            &scene,
            4096,
            "PT_microfacet_test.ppm",
        )?;
    }

    Ok(())
}
