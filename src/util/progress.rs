use crate::parallel::SpinLock;

#[derive(Default)]
struct ProgressContext {
    current: usize,
    percent: i32,
    last_percent: i32,
}

pub struct Progress {
    total: usize,
    step: usize,

    context: SpinLock<ProgressContext>,
}

impl Progress {
    pub fn new(total: usize, step: usize) -> Self {
        println!("0%");
        Self {
            total,
            step,
            context: SpinLock::new(Default::default()),
        }
    }

    pub fn update(&self, count: usize) {
        let mut context = self.context.lock();

        context.current += count;
        context.percent = (100.0 * context.current as f32 / self.total as f32) as i32;
        if (context.percent - context.last_percent) as usize >= self.step {
            context.last_percent = context.percent;
            println!("{}%", context.percent);
        }
    }
}
