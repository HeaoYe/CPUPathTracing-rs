use cpu_path_tracing::{camera, geometry, integrator, light_sampler, material, scene};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 10, 108 * 10);
    let camera_pos = glam::vec3(0.0, 37.0, -61.0);
    let mut camera = camera::Camera::new(film, camera_pos, glam::vec3(0.0, 8.0, 0.0), 16.0);

    let mut builder = scene::SceneBuilder::default();

    let triangles = [
        geometry::Triangle::from_points(
            glam::vec3(-17.0, 0.0, -1.5),
            glam::vec3(-17.0, 0.0, 1.5),
            glam::vec3(17.0, 0.0, 1.5),
        ),
        geometry::Triangle::from_points(
            glam::vec3(-17.0, 0.0, -1.5),
            glam::vec3(17.0, 0.0, 1.5),
            glam::vec3(17.0, 0.0, -1.5),
        ),
    ];
    let light_sphere_1 = geometry::Sphere {
        center: glam::vec3(-15.0 + 00.0 / 3.0, 12.0, 8.0),
        radius: 2.0,
    };
    let light_sphere_2 = geometry::Sphere {
        center: glam::vec3(-15.0 + 30.0 / 3.0, 12.0, 8.0),
        radius: 1.0,
    };
    let light_sphere_3 = geometry::Sphere {
        center: glam::vec3(-15.0 + 60.0 / 3.0, 12.0, 8.0),
        radius: 0.5,
    };
    let light_sphere_4 = geometry::Sphere {
        center: glam::vec3(-15.0 + 90.0 / 3.0, 12.0, 8.0),
        radius: 0.1,
    };
    builder.add_area_light(
        &light_sphere_1,
        material::Material::default(),
        scene::InstanceTransform::default(),
        glam::Vec3::splat(1.0),
        false,
    );
    builder.add_area_light(
        &light_sphere_2,
        material::Material::default(),
        scene::InstanceTransform::default(),
        glam::Vec3::splat(4.0),
        false,
    );
    builder.add_area_light(
        &light_sphere_3,
        material::Material::default(),
        scene::InstanceTransform::default(),
        glam::Vec3::splat(16.0),
        false,
    );
    builder.add_area_light(
        &light_sphere_4,
        material::Material::default(),
        scene::InstanceTransform::default(),
        glam::Vec3::splat(400.0),
        false,
    );
    let light_pos_center = glam::vec3(0.0, 12.0, 8.0);
    let alphas = [0.4, 0.25, 0.16, 0.04];
    for (i, &alpha) in alphas.iter().enumerate() {
        let theta = (i as f32 * 15.0).to_radians();
        let center = glam::vec3(0.0, 17.0 * (1.0 - theta.cos()), 17.0 * theta.sin());
        let normal = ((light_pos_center - center).normalize() + (camera_pos - center).normalize())
            .normalize();
        let rotation_x = -normal.y.acos();
        builder.add_shape(
            &triangles[0],
            material::Material::conductor_with_alpha(
                [2.0, 2.0, 1.0],
                [3.0, 3.0, 15.0],
                alpha,
                alpha,
            ),
            scene::InstanceTransform {
                translation: center,
                rotation: glam::Quat::from_rotation_x(rotation_x),
                ..Default::default()
            },
        );
        builder.add_shape(
            &triangles[1],
            material::Material::conductor_with_alpha(
                [2.0, 2.0, 1.0],
                [3.0, 3.0, 15.0],
                alphas[i],
                alphas[i],
            ),
            scene::InstanceTransform {
                translation: center,
                rotation: glam::Quat::from_rotation_x(rotation_x),
                ..Default::default()
            },
        );
    }

    let ground = geometry::Plane::new(glam::vec3(0.0, -5.0, 0.0), glam::Vec3::Y, 100.0);
    let wall = geometry::Plane::new(glam::vec3(0.0, 0.0, 15.0), glam::Vec3::NEG_Z, 100.0);
    builder.add_shape(
        &ground,
        material::Material::ground(glam::Vec3::ONE),
        Default::default(),
    );
    builder.add_shape(
        &wall,
        material::Material::diffuse(glam::Vec3::ONE),
        Default::default(),
    );
    builder.add_uniform_infinite_light([0.5, 0.5, 0.5]);

    let scene = builder.build();

    let mixture_light_sampler = light_sampler::LightSampler::<
        light_sampler::MixtureLightSelector<
            20,
            light_sampler::UniformLightSelector,
            light_sampler::PowerLightSelector,
        >,
    >::new(&scene, light_sampler::MisCompensation::Disabled);

    let light_sampler_compensated = light_sampler::LightSampler::<
        light_sampler::MixtureLightSelector<
            20,
            light_sampler::UniformLightSelector,
            light_sampler::PowerLightSelector,
        >,
    >::new(&scene, light_sampler::MisCompensation::Enabled);

    let simple_path_tracing_integrator =
        integrator::SimplePathTracingIntegrator::new(&mixture_light_sampler);

    let path_tracing_integrator = integrator::PathTracingIntegrator::new(&mixture_light_sampler);

    let path_tracing_integrator_compensated =
        integrator::PathTracingIntegrator::new(&light_sampler_compensated);

    if integrator::preview(&path_tracing_integrator_compensated, &mut camera, &scene) {
        integrator::render(
            &simple_path_tracing_integrator,
            &mut camera,
            &scene,
            64,
            "PT_NEE.ppm",
        )?;
        integrator::render(
            &path_tracing_integrator,
            &mut camera,
            &scene,
            64,
            "PT_MIS.ppm",
        )?;
        integrator::render(
            &path_tracing_integrator_compensated,
            &mut camera,
            &scene,
            64,
            "PT_MIS_COMPENSATION.ppm",
        )?;
    }

    Ok(())
}
