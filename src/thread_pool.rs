use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct WorkerContext {
    alive: AtomicBool,
    tasks: Mutex<VecDeque<Task>>,
    pending_task_count: AtomicUsize,
}

pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    context: Arc<WorkerContext>,
}

impl ThreadPool {
    pub fn new(mut thread_count: usize) -> Self {
        if thread_count == 0 {
            thread_count = match thread::available_parallelism() {
                Ok(count) => count.get(),
                _ => 1,
            }
        };

        let context = Arc::new(WorkerContext {
            alive: AtomicBool::new(true),
            tasks: Mutex::new(VecDeque::new()),
            pending_task_count: AtomicUsize::new(0),
        });

        let mut workers = Vec::with_capacity(thread_count);
        for i in 0..thread_count {
            let context = Arc::clone(&context);
            workers.push(
                thread::Builder::new()
                    .name(format!("worker-{}", i))
                    .spawn(|| Self::worker(context))
                    .unwrap(),
            );
        }

        Self { workers, context }
    }

    pub fn add_task(&mut self, task: Task) {
        let mut task_deque = self.context.tasks.lock().unwrap();
        self.context
            .pending_task_count
            .fetch_add(1, Ordering::Relaxed);
        task_deque.push_back(task);
    }

    pub fn wait(&self) {
        while self.context.pending_task_count.load(Ordering::Acquire) != 0 {
            std::hint::spin_loop();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.wait();
        self.context.alive.store(false, Ordering::Relaxed);
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

struct CompletionGuard<'a> {
    counter: &'a AtomicUsize,
}

impl<'a> Drop for CompletionGuard<'a> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Release);
    }
}

impl ThreadPool {
    fn worker(context: Arc<WorkerContext>) {
        while context.alive.load(Ordering::Relaxed) {
            if let Some(task) = {
                let mut task_deque = context.tasks.lock().unwrap();
                task_deque.pop_front()
            } {
                let _guard = CompletionGuard {
                    counter: &context.pending_task_count,
                };
                task();
            } else {
                std::hint::spin_loop();
            }
        }
    }
}
