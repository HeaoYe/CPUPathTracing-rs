use super::spin_lock::SpinLock;
use std::{
    collections::VecDeque,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};

type ScopeTask<'a> = Box<dyn FnOnce() + Send + 'a>;
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

    pub fn workers(&self) -> usize {
        self.workers.len()
    }

    pub fn add_task(&self, task: Task) {
        let mut task_deque = self.context.tasks.lock();
        task_deque.push_back(task);
        self.context
            .pending_task_count
            .fetch_add(1, Ordering::Relaxed);
    }

    // SAFETY: 调用者必须使用 THREAD_POOL.wait() 确保任务的生命周期满足要求。
    pub unsafe fn add_scope_task_unchecked(&self, task: ScopeTask) {
        let task: Task = unsafe { std::mem::transmute(task) };
        self.add_task(task);
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

struct TileMut<'a, T> {
    ptr: std::ptr::NonNull<T>,
    total_width: usize,

    x: usize,
    y: usize,
    tile_width: usize,

    _marker: PhantomData<&'a mut T>,
}

impl<T> TileMut<'_, T> {
    unsafe fn new(
        ptr: std::ptr::NonNull<T>,
        total_width: usize,

        x: usize,
        y: usize,
        tile_width: usize,
    ) -> Self {
        Self {
            ptr,
            total_width,
            x,
            y,
            tile_width,
            _marker: PhantomData,
        }
    }

    fn row_mut(&mut self, y: usize) -> &mut [T] {
        let offset = (self.y + y) * self.total_width + self.x;

        // SAFETY: parallel_for_2d 函数负责分割出互不重叠的 tiles
        // 且所有 tile 会在 scope 结束前使用完毕
        // 因此可以安全的将 ptr 转换为 &mut slice
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr().add(offset), self.tile_width) }
    }
}

// SAFETY: parallel_for_2d 函数负责分割出互不重叠的 tiles
// 因此每份 tile 都可以 Send
unsafe impl<T: Send> Send for TileMut<'_, T> {}

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
    #[must_use]
    unsafe fn dispatch_scope_tasks<'a>(
        &self,
        tasks: Vec<ScopeTask<'_>>,
        scope_counter: &'a AtomicUsize,
    ) -> ScopeWaitGuard<'a> {
        let task_count = tasks.len();
        let mut task_deque = self.context.tasks.lock();
        task_deque.reserve(task_count);

        // SAFETY: ScoopWaitGuard 保证离开作用域前所有任务执行完毕
        // 并且 task 只借用了作用域内的 func
        // 故可以安全的将生命周期转换为 'static
        // note: type Task = Box<dyn FnOnce() + Send + 'static>
        let tasks: Vec<Task> = unsafe { std::mem::transmute(tasks) };
        task_deque.extend(tasks);
        self.context
            .pending_task_count
            .fetch_add(task_count, Ordering::Relaxed);
        scope_counter.fetch_add(task_count, Ordering::Relaxed);

        ScopeWaitGuard { scope_counter }
    }

    fn parallel_for_1d_impl<T, F>(&self, tasks_per_worker: usize, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Sync,
    {
        if data.is_empty() {
            return;
        }

        let chunk_size = data
            .len()
            .div_ceil(self.workers.len() * tasks_per_worker)
            .max(1);
        let task_count = data.len().div_ceil(chunk_size);

        let mut scope_tasks: Vec<ScopeTask<'_>> = Vec::with_capacity(task_count);
        let scope_counter = AtomicUsize::new(0);

        let scope_counter = &scope_counter;
        let func = &func;

        for (chunk_idx, chunk_data) in data.chunks_mut(chunk_size).enumerate() {
            scope_tasks.push(Box::new(move || {
                let _guard = CompletionGuard {
                    counter: scope_counter,
                };
                for (local_x, value) in chunk_data.iter_mut().enumerate() {
                    let x = chunk_idx * chunk_size + local_x;
                    func(x, value);
                }
            }));
        }

        // SAFETY: 使用 _scope_wait_guard 保证作用域结束前所有 ScopeTask 运行完毕
        let _scope_wait_guard = unsafe { self.dispatch_scope_tasks(scope_tasks, scope_counter) };
    }

    fn parallel_for_2d_impl<T, F>(
        &self,
        width: usize,
        height: usize,
        tasks_per_worker: usize,
        data: &mut [T],
        func: F,
    ) where
        T: Send,
        F: Fn(usize, usize, &mut T) + Sync,
    {
        if data.is_empty() || data.len() != width * height {
            return;
        }

        let base_ptr = std::ptr::NonNull::new(data.as_mut_ptr()).unwrap();
        let tile_area = (width * height).div_ceil(self.workers.len() * tasks_per_worker) as f64;
        let tile_side = tile_area.sqrt().ceil() as usize;
        let task_count = width.div_ceil(tile_side) * height.div_ceil(tile_side);

        let mut scope_tasks: Vec<ScopeTask<'_>> = Vec::with_capacity(task_count);
        let scope_counter = AtomicUsize::new(0);

        let scope_counter = &scope_counter;
        let func = &func;

        for tile_y in (0..height).step_by(tile_side) {
            for tile_x in (0..width).step_by(tile_side) {
                let w = tile_side.min(width - tile_x);
                let h = tile_side.min(height - tile_y);

                // SAFETY: 此处的实现方式可以保证各个 tile 互不重叠
                let mut tile = unsafe { TileMut::new(base_ptr, width, tile_x, tile_y, w) };

                scope_tasks.push(Box::new(move || {
                    let _guard = CompletionGuard {
                        counter: scope_counter,
                    };
                    for local_y in 0..h {
                        let y = tile_y + local_y;
                        for (local_x, value) in tile.row_mut(local_y).iter_mut().enumerate() {
                            let x = tile_x + local_x;
                            func(x, y, value);
                        }
                    }
                }));
            }
        }

        // SAFETY: 使用 _scope_wait_guard 保证作用域结束前所有 ScopeTask 运行完毕
        let _scope_wait_guard = unsafe { self.dispatch_scope_tasks(scope_tasks, scope_counter) };
    }
}

impl ThreadPool {
    pub fn parallel_for_1d<T, F>(&self, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Sync,
    {
        self.parallel_for_1d_impl(16, data, func);
    }

    pub fn parallel_for_1d_coarse<T, F>(&self, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Sync,
    {
        self.parallel_for_1d_impl(1, data, func);
    }

    pub fn parallel_for_2d<T, F>(&self, width: usize, height: usize, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, usize, &mut T) + Sync,
    {
        self.parallel_for_2d_impl(width, height, 16, data, func);
    }

    pub fn parallel_for_2d_coarse<T, F>(&self, width: usize, height: usize, data: &mut [T], func: F)
    where
        T: Send,
        F: Fn(usize, usize, &mut T) + Sync,
    {
        self.parallel_for_2d_impl(width, height, 1, data, func);
    }
}

pub static THREAD_POOL: std::sync::LazyLock<ThreadPool> =
    std::sync::LazyLock::new(|| ThreadPool::new(0));
