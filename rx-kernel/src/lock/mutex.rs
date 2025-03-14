use core::cell::{Cell, UnsafeCell};
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering, fence};

use crate::process::cpu::{cpuid, pop_off, push_off};
use crate::{STARTED, println};

#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized> {
    locked: AtomicBool,
    name: &'static str,
    cpu_id: Cell<isize>,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T> {
    spinlock: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(data: T, name: &'static str) -> Self {
        let lock = Mutex {
            locked: AtomicBool::new(false),
            name,
            cpu_id: Cell::new(-1),
            data: UnsafeCell::new(data),
        };
        lock
    }

    pub unsafe fn raw_data_mut_unchecked(&self) -> *mut T {
        self.data.get()
    }

    pub fn as_ptr(&self) -> *mut bool {
        self.locked.as_ptr()
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        push_off();
        if self.holding() {
            panic!("spinlock {} acquire", self.name);
        }

        while self.locked.swap(true, Ordering::Acquire) {
            // Now we signals the processor that it is inside a busy-wait spin-loop
            spin_loop();
        }
        fence(Ordering::SeqCst);
        unsafe {
            self.cpu_id.set(cpuid() as isize);
        }

        MutexGuard { spinlock: self }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        push_off();
        if !self.holding() && !self.locked.swap(true, Ordering::Acquire) {
            fence(Ordering::SeqCst);
            unsafe {
                self.cpu_id.set(cpuid() as isize);
            }
            Some(MutexGuard { spinlock: self })
        } else {
            pop_off();
            None
        }
    }

    pub fn force_unlock(&self) {
        self.release();
    }

    pub fn release(&self) {
        if !self.holding() {
            panic!("spinlock {} release", self.name);
        }
        self.cpu_id.set(-1);
        fence(Ordering::SeqCst);
        self.locked.store(false, Ordering::Release);
        pop_off();
    }

    // Check whether this cpu is holding the lock.
    // Interrupts must be off.
    pub fn holding(&self) -> bool {
        // self.locked.load(Ordering::Relaxed) && (self.cpu_id.get() == unsafe{ cpuid() } as isize)
        if self.locked.load(Ordering::Relaxed) && self.cpu_id.get() == unsafe { cpuid() } as isize {
            return true;
        }
        false
    }
}

impl<'a, T> MutexGuard<'a, T> {
    pub unsafe fn holding(&self) -> bool {
        self.spinlock.holding()
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.spinlock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.spinlock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.spinlock.release()
    }
}

// We need to force Send and Sync traits because our mutex has
// UnsafeCell, which don't realize it
// As long as T: Send, it's fine to send and share Mutex<T> between threads

unsafe impl<T> Send for Mutex<T> where T: Send {}
unsafe impl<T> Sync for Mutex<T> where T: Send {}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Send + Sync {}
