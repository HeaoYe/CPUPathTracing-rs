mod debug_integrator;
mod normal_integrator;
mod simple_rt_integrator;

pub use debug_integrator::{BoundsTestIntegrator, BvhDepthIntegrator, TriangleTestIntegrator};
pub use normal_integrator::NormalIntegrator;
pub use simple_rt_integrator::SimpleRTIntegrator;

use crate::{
    THREAD_POOL,
    camera::{Camera, CameraModel, PixelSample},
    scene::Scene,
    util::{Progress, profile},
};

pub trait Integrator {
    fn integrate(
        &self,
        x: usize,
        y: usize,
        sample_index: usize,
        camera: &CameraModel,
        scene: &Scene,
    ) -> Option<PixelSample>;
}

pub fn render<T>(
    integrator: &T,
    camera: &mut Camera,
    scene: &Scene,
    spp: usize,
    filename: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error>
where
    T: Integrator + Sync,
{
    profile!("render {} spp {}", spp, filename.as_ref().display());

    let Camera { film, geometry } = camera;
    film.clear();

    let mut current_spp = 0;
    let mut increase = 1;
    let progress = Progress::new(film.width() * film.height() * spp, 20);
    let filename = filename.as_ref();

    while current_spp < spp {
        let batch_spp = increase.min(spp - current_spp);
        THREAD_POOL.parallel_for_2d(
            film.width(),
            film.height(),
            film.as_slice_mut(),
            |x, y, pixel| {
                for i in 0..batch_spp {
                    if let Some(sample) =
                        integrator.integrate(x, y, current_spp + i, geometry, scene)
                    {
                        pixel.add_sample(sample);
                    }
                }
                progress.update(batch_spp);
            },
        );

        current_spp += batch_spp;
        increase = current_spp.min(32);

        film.save(filename)?;
        println!(
            "{} spp has been saved to {}",
            current_spp,
            filename.display()
        );
    }

    Ok(())
}
