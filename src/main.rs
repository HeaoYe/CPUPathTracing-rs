mod film;
mod thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut film = film::Film::new(1920, 1080);
    for i in 0..100 {
        for j in 0..200 {
            film.set_pixel(j, i, glam::Vec3::new(0.5, 0.7, 0.2));
        }
    }
    film.save("test.ppm")?;

    let mut thread_pool = thread_pool::ThreadPool::new(0);
    let task = || println!("[{}] Hello World !", std::thread::current().name().unwrap());
    thread_pool.add_task(Box::new(task));
    thread_pool.add_task(Box::new(task));
    thread_pool.add_task(Box::new(task));
    thread_pool.add_task(Box::new(task));

    Ok(())
}
