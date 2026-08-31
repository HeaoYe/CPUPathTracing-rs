use std::time::Instant;

pub(crate) struct Profile {
    name: String,
    start: Instant,
}

impl Profile {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        println!(
            "Profile \"{}\": {:?} ms",
            self.name,
            self.start.elapsed().as_micros() as f64 / 1e3
        );
    }
}

macro_rules! profile {
    ($($arg:tt)*) => {
        let _profile_guard = $crate::util::Profile::new(format!($($arg)*));
    };
}

pub(crate) use profile;
