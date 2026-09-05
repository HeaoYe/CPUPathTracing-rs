use cpu_path_tracing::{
    camera, color, geometry, image, integrator, light_sampler, material, sample, scene, spectrum,
    util,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let film = camera::Film::new(192 * 10, 108 * 10);
    let mut camera = camera::Camera::new(
        film,
        glam::vec3(0.0, 1.25, -10.0),
        glam::vec3(0.0, 3.95, 2.0),
        48.0,
    );

    let mut builder = scene::SceneBuilder::default();

    let buddha = geometry::Model::load("models/buddha.obj")?;
    let dragon = geometry::Model::load("models/dragon_871k.obj")?;

    let copper_eta = spectrum::Spectrum::piecewise_linear_from_csv(
        "spectrums/Johnson-copper.csv",
        "wl",
        1e3,
        "n",
    )?;
    let copper_k = spectrum::Spectrum::piecewise_linear_from_csv(
        "spectrums/Johnson-copper.csv",
        "wl",
        1e3,
        "k",
    )?;
    let copper = material::Material::conductor_with_alpha(&copper_eta, &copper_k, 0.5, 0.42);
    let glass_eta = spectrum::Spectrum::piecewise_linear_from_csv(
        "spectrums/Zelmon-glass.csv",
        "wl",
        1e3,
        "n",
    )?;
    let glass_tint = color::LUT_SRGB.lookup_rgb8(210, 210, 184);
    let glass = material::Material::dielectric_with_alpha_tint(&glass_eta, &glass_tint, 0.21, 0.08);

    builder.add_shape(
        &buddha,
        copper,
        scene::InstanceTransform {
            translation: glam::vec3(-5.0, 1.78, 1.7),
            scale: glam::Vec3::splat(4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &buddha,
        glass,
        scene::InstanceTransform {
            translation: glam::vec3(-2.1, 1.78, 1.7),
            scale: glam::Vec3::splat(4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &buddha,
        glass,
        scene::InstanceTransform {
            translation: glam::vec3(2.1, 1.78, 1.7),
            scale: glam::Vec3::splat(4.0),
            ..Default::default()
        },
    );
    builder.add_shape(
        &buddha,
        copper,
        scene::InstanceTransform {
            translation: glam::vec3(5.0, 1.78, 1.7),
            scale: glam::Vec3::splat(4.0),
            ..Default::default()
        },
    );

    let sphere = geometry::Sphere {
        center: glam::Vec3::ZERO,
        radius: 0.45,
    };
    let sphere_eta = spectrum::Spectrum::piecewise_linear_from_samples([
        spectrum::SamplePoint {
            lambda: 360.0,
            value: 1.60,
        },
        spectrum::SamplePoint {
            lambda: 400.0,
            value: 1.57,
        },
        spectrum::SamplePoint {
            lambda: 525.0,
            value: 1.52,
        },
        spectrum::SamplePoint {
            lambda: 650.0,
            value: 1.48,
        },
        spectrum::SamplePoint {
            lambda: 830.0,
            value: 1.45,
        },
    ]);
    let sphere_mat = material::Material::dielectric_with_alpha_tint(
        &sphere_eta,
        &spectrum::Spectrum::Default,
        0.0,
        0.0,
    );

    builder.add_shape(
        &sphere,
        sphere_mat,
        scene::InstanceTransform {
            translation: glam::vec3(-1.0, 0.55, -1.65),
            ..Default::default()
        },
    );
    builder.add_shape(
        &sphere,
        sphere_mat,
        scene::InstanceTransform {
            translation: glam::vec3(1.0, 0.55, -1.65),
            ..Default::default()
        },
    );

    let metameric_a_reflectance = spectrum::Spectrum::dense_from_csv(
        "spectrums/Metameric_A_reflectance.csv",
        "wavelength_nm",
        "reflectance",
    )?;
    let metameric_a = material::Material::diffuse(&metameric_a_reflectance);
    let metameric_b_reflectance = spectrum::Spectrum::dense_from_csv(
        "spectrums/Metameric_B_reflectance.csv",
        "wavelength_nm",
        "reflectance",
    )?;
    let metameric_b = material::Material::diffuse(&metameric_b_reflectance);

    builder.add_shape(
        &dragon,
        metameric_a,
        scene::InstanceTransform {
            translation: glam::vec3(-4.3, 0.7, -1.65),
            scale: glam::Vec3::splat(2.0),
            rotation: glam::Quat::from_rotation_y((-90.0_f32).to_radians()),
        },
    );
    builder.add_shape(
        &dragon,
        metameric_b,
        scene::InstanceTransform {
            translation: glam::vec3(-2.4, 0.7, -1.65),
            scale: glam::Vec3::splat(2.0),
            rotation: glam::Quat::from_rotation_y((-90.0_f32).to_radians()),
        },
    );
    builder.add_shape(
        &dragon,
        metameric_b,
        scene::InstanceTransform {
            translation: glam::vec3(2.4, 0.7, -1.65),
            scale: glam::Vec3::splat(2.0),
            rotation: glam::Quat::from_rotation_y(90.0_f32.to_radians()),
        },
    );
    builder.add_shape(
        &dragon,
        metameric_a,
        scene::InstanceTransform {
            translation: glam::vec3(4.3, 0.7, -1.65),
            scale: glam::Vec3::splat(2.0),
            rotation: glam::Quat::from_rotation_y(90.0_f32.to_radians()),
        },
    );

    const DRAGON_COUNT: usize = 1200;
    let mut rng = util::Rng::new(DRAGON_COUNT as u64, 0);
    let reflectance_spectral: Vec<_> = std::iter::repeat_with(|| {
        color::LUT_SRGB.lookup_linear(color::LinearRgb::new(
            rng.uniform(),
            rng.uniform(),
            rng.uniform(),
        ))
    })
    .take(DRAGON_COUNT)
    .collect();

    for spectral in &reflectance_spectral {
        let disk = sample::uniform::disk(rng.uniform(), rng.uniform()) * 12.0;

        builder.add_shape(
            &dragon,
            material::Material::diffuse(spectral),
            scene::InstanceTransform {
                translation: glam::vec3(disk.x, disk.y.abs(), 4.0),
                rotation: glam::Quat::from_euler(
                    glam::EulerRot::ZYX,
                    (rng.uniform() * 360.0).to_radians(),
                    (rng.uniform() * 360.0).to_radians(),
                    (rng.uniform() * 360.0).to_radians(),
                ),
                ..Default::default()
            },
        );
    }

    let ground = geometry::Plane::new(glam::Vec3::ZERO, glam::Vec3::Y, 100.0);
    builder.add_shape(
        &ground,
        material::Material::ground(&spectrum::Spectrum::Default),
        Default::default(),
    );

    let env_image = image::RgbImage::load_exr("hdris/kloppenheim_07_puresky_4k.exr")?;
    builder.add_image_infinite_light(&env_image, 80.0);
    // let uniform_light = spectrum::Spectrum::rgb_illuminant_linear_rgb(
    //     &color::LUT_SRGB,
    //     LinearRgb::new(1.0, 0.9, 0.8),
    // );
    // builder.add_uniform_infinite_light(&uniform_light);

    let scene = builder.build();

    let light_sampler = light_sampler::LightSampler::<
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
        integrator::SimplePathTracingIntegrator::new(&light_sampler);
    let path_tracing_integrator =
        integrator::PathTracingIntegrator::new(&light_sampler_compensated);

    if integrator::preview(&path_tracing_integrator, &mut camera, &scene) {
        integrator::render(
            &path_tracing_integrator,
            &mut camera,
            &scene,
            64,
            "RGB2SPECTRAL_TEST_64.exr",
            &color::SRGB,
        )?;
        integrator::render(
            &simple_path_tracing_integrator,
            &mut camera,
            &scene,
            64,
            "RGB2SPECTRAL_TEST_64_SIMPLE.exr",
            &color::SRGB,
        )?;
    }

    Ok(())
}
