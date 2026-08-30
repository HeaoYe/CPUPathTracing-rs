mod film;
mod spin_lock;
mod thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_pool = thread_pool::ThreadPool::new(0);

    let mut film = film::Film::new(1920, 1080);

    thread_pool.parallel_for_2d(
        film.width(),
        film.height(),
        film.as_slice_mut(),
        |x, y, pixel| {
            if x < 200 && y < 100 {
                *pixel = glam::Vec3::new(0.5, 0.7, 0.2);
            }
        },
    );

    film.save("test.ppm")?;

    Ok(())
}
