use cpu_path_tracing::{
    camera, color, geometry, image, integrator, light_sampler, material, scene, spectrum,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eew = spectrum::Spectrum::constant(1.0);
    let xyz_eew = color::Xyz::from_spectrum(&eew);
    println!("EEW XYZ: {:?}", xyz_eew);

    let xyz_d65 = color::Xyz::from_spectrum(&spectrum::CIE_STD_ILLUMNT_D65);
    let chroma_d65 = color::Chromaticity::from(xyz_d65);
    println!("D65 Chroma: {:?}", chroma_d65);

    let blackbody_6504k = spectrum::Spectrum::blackbody(6504.0);
    let chroma_6504k = color::Chromaticity::from(color::Xyz::from_spectrum(&blackbody_6504k));
    println!("6504k Chroma: {:?}", chroma_6504k);

    let rgb_d65_srgb = color::SRGB.rgb_from_xyz(xyz_d65);
    let rgb_d65_dci_p3 = color::DCI_P3.rgb_from_xyz(xyz_d65);
    println!("sRGB D65: {:?}", rgb_d65_srgb);
    println!("DCI-P3 D65: {:?}", rgb_d65_dci_p3);

    let d65_600nit = spectrum::Spectrum::illuminant(&spectrum::CIE_STD_ILLUMNT_D65, 600.0);
    let rgb_d65_600nit_srgb = color::SRGB.rgb_from_xyz(color::Xyz::from_spectrum(&d65_600nit));
    println!("sRGB D65 600nit: {:?}", rgb_d65_600nit_srgb);

    let encoded_rgb_d65_600nit_srgb = color::SRGB.encode(rgb_d65_600nit_srgb);
    println!(
        "sRGB D65 600nit: {:?}",
        encoded_rgb_d65_600nit_srgb.to_quantized(8)
    );

    let film = camera::Film::new(192 * 10, 108 * 10);
    let mut camera = camera::Camera::new(
        film,
        glam::vec3(0.0, 1.25, -6.0),
        glam::vec3(0.0, 1.95, 0.0),
        45.0,
    );

    let mut builder = scene::SceneBuilder::default();

    let model = geometry::Model::load("models/buddha.obj")?;

    let rgb = |r: u8, g: u8, b: u8| {
        color::SRGB.decode(color::EncodedRgb::from_quantized(
            r as u32, g as u32, b as u32, 8,
        ))
    };
    builder.add_shape(
        &model,
        material::Material::specular(rgb(241, 191, 79)),
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
            rgb(180, 180, 154),
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
        material::Material::diffuse(rgb(241, 191, 79)),
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

    // builder.add_uniform_infinite_light([0.5, 0.5, 0.5]);
    // let env_image =
    //     image::RgbImage::load_exr("hdris/HdrOutdoorSnowMountainsEveningClear001_HDR_4K.exr")?;
    // let env_image = image::RgbImage::load_exr("hdris/qwantani_night_puresky_4k.exr")?;
    let env_image = image::RgbImage::load_exr("hdris/kloppenheim_07_puresky_4k.exr")?;
    builder.add_image_infinite_light(&env_image, 0.0);

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
            "PT_MIS_sRGB.exr",
            &color::SRGB,
        )?;
        camera.film.save("PT_MIS_DCI_P3.exr", &color::DCI_P3)?;
    }

    Ok(())
}
