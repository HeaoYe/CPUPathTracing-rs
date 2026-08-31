use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
    flag: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            flag: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn acquire(&self) {
        loop {
            if self
                .flag
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }

            while self.flag.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn release(&self) {
        self.flag.store(false, Ordering::Release);
    }
}

pub struct SpinLockGuard<'a, T> {
    spin_lock: &'a SpinLock<T>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.spin_lock.release();
    }
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: SpinLock 保证同一时间最多只有一个 SpinLockGuard
        unsafe { &*self.spin_lock.value.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: SpinLock 保证同一时间最多只有一个 SpinLockGuard
        unsafe { &mut *self.spin_lock.value.get() }
    }
}

impl<T> SpinLock<T> {
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        self.acquire();
        SpinLockGuard {
            spin_lock: self,
            _marker: PhantomData,
        }
    }
}

// SAFETY: T 只通过持有 SpinLockGuard 时访问
// SpinLock 保证同一时间只有一个线程持有 SpinLockGuard
// 因此对 T 的访问不会并发发生
// 故 T: Send 足以使 SpinLock<T>: Sync
unsafe impl<T: Send> Sync for SpinLock<T> {}
