use cpu_path_tracing::{camera, geometry, integrator, light_sampler, material, scene, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 10, 108 * 10);
    let mut camera = camera::Camera::new(
        film,
        glam::vec3(0.0, 1.25, -6.0),
        glam::vec3(0.0, 1.95, 0.0),
        45.0,
    );

    let mut builder = scene::SceneBuilder::default();

    let model = geometry::Model::load("models/buddha.obj")?;
    builder.add_shape(
        &model,
        material::Material::specular(util::Rgb::new(241, 191, 79)),
        scene::InstanceTransform {
            translation: glam::vec3(-3.0, 1.75, 0.0),
            scale: glam::vec3(4.0, 4.0, 4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &model,
        material::Material::conductor_with_alpha([1.2, 1.2, 5.3], [3.4, 3.4, 2.1], 0.8, 0.2),
        scene::InstanceTransform {
            translation: glam::vec3(-1.0, 1.75, 0.0),
            scale: glam::vec3(4.0, 4.0, 4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &model,
        material::Material::dielectric_with_alpha(
            1.4,
            glam::Vec3::ONE,
            util::Rgb::new(180, 180, 154),
            0.1,
            0.3,
        ),
        scene::InstanceTransform {
            translation: glam::vec3(1.0, 1.75, 0.0),
            scale: glam::vec3(4.0, 4.0, 4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &model,
        material::Material::diffuse(util::Rgb::new(241, 191, 79)),
        scene::InstanceTransform {
            translation: glam::vec3(3.0, 1.75, 0.0),
            scale: glam::vec3(4.0, 4.0, 4.0),
            ..Default::default()
        },
    );

    let sphere = geometry::Sphere {
        center: glam::Vec3::ZERO,
        radius: 1.0,
    };
    builder.add_shape(
        &sphere,
        material::Material::specular(glam::Vec3::ONE),
        scene::InstanceTransform {
            translation: glam::vec3(0.0, 3.75, 3.0),
            ..Default::default()
        },
    );

    let ground = geometry::Plane::new(glam::Vec3::ZERO, glam::Vec3::Y, 100.0);
    builder.add_shape(
        &ground,
        material::Material::ground(glam::Vec3::ONE),
        Default::default(),
    );

    builder.add_uniform_infinite_light([0.5, 0.5, 0.5]);

    let scene = builder.build();

    let light_sampler_compensated = light_sampler::LightSampler::<
        light_sampler::MixtureLightSelector<
            20,
            light_sampler::UniformLightSelector,
            light_sampler::PowerLightSelector,
        >,
    >::new(&scene, light_sampler::MisCompensation::Enabled);

    let path_tracing_integrator =
        integrator::PathTracingIntegrator::new(&light_sampler_compensated);

    if integrator::preview(&path_tracing_integrator, &mut camera, &scene) {
        integrator::render(
            &path_tracing_integrator,
            &mut camera,
            &scene,
            64,
            "PT_MIS.ppm",
        )?;
    }

    Ok(())
}
