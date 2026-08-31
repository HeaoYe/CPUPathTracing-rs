use super::spin_lock::{SpinLock, SpinLockGuard};
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct WorkerContext {
    alive: AtomicBool,
    tasks: SpinLock<VecDeque<Task>>,
    pending_task_count: AtomicUsize,
}

impl WorkerContext {
    fn wait(&self) {
        while self.pending_task_count.load(Ordering::Acquire) != 0 {
            std::thread::sleep(std::time::Duration::from_millis(2));
            std::hint::spin_loop();
        }
    }
}

pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    context: Arc<WorkerContext>,
}

impl ThreadPool {
    pub fn new(mut thread_count: usize) -> Self {
        if thread_count == 0 {
            thread_count = thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1);
        }

        let context = Arc::new(WorkerContext {
            alive: AtomicBool::new(true),
            tasks: SpinLock::new(VecDeque::new()),
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

    pub fn add_task(&self, task: Task) {
        let mut task_deque = self.context.tasks.lock();
        self.add_task_private(task, &mut task_deque);
    }

    pub fn add_task_private(&self, task: Task, task_deque: &mut SpinLockGuard<VecDeque<Task>>) {
        task_deque.push_back(task);
        self.context
            .pending_task_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn wait(&self) {
        self.context.wait();
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
                let mut task_deque = context.tasks.lock();
                task_deque.pop_front()
            } {
                let _guard = CompletionGuard {
                    counter: &context.pending_task_count,
                };
                task();
            } else {
                std::thread::sleep(std::time::Duration::from_millis(2));
                std::hint::spin_loop();
            }
        }
    }
}

struct ScopeWaitGuard<'a> {
    scope_counter: &'a AtomicUsize,
}

impl<'a> Drop for ScopeWaitGuard<'a> {
    fn drop(&mut self) {
        while self.scope_counter.load(Ordering::Acquire) != 0 {
            std::thread::sleep(std::time::Duration::from_millis(2));
            std::hint::spin_loop();
        }
    }
}

impl ThreadPool {
    pub fn parallel_for_2d<T, F>(&self, width: usize, height: usize, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, usize, &mut T) + Sync,
    {
        if data.len() != width * height {
            return;
        }

        let scope_counter = AtomicUsize::new(0);

        let _guard = ScopeWaitGuard {
            scope_counter: &scope_counter,
        };

        {
            let mut task_deque = self.context.tasks.lock();
            let func = &func;
            let scope_counter = &scope_counter;
            for (y, line) in data.chunks_mut(width).enumerate() {
                let task: Box<dyn FnOnce() + Send + '_> = Box::new(move || {
                    let _guard = CompletionGuard {
                        counter: scope_counter,
                    };
                    for (x, value) in line.iter_mut().enumerate() {
                        func(x, y, value);
                    }
                });

                // SAFETY: ScoopWaitGuard 保证离开作用域前所有任务执行完毕
                // 并且 task 只借用了作用域内的 func
                // 故可以安全的将生命周期转换为 'static
                // note: type Task = Box<dyn FnOnce() + Send + 'static>
                let task: Box<dyn FnOnce() + Send + 'static> = unsafe { std::mem::transmute(task) };
                self.add_task_private(task, &mut task_deque);
                scope_counter.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

pub static THREAD_POOL: std::sync::LazyLock<ThreadPool> =
    std::sync::LazyLock::new(|| ThreadPool::new(0));
