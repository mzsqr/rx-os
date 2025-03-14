use core::{
    cell::{Cell, UnsafeCell},
    fmt,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering, fence},
};

use crate::process::cpu::{self, pop_off, push_off};

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    name: &'static str,
    cpu_id: Cell<isize>,
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    pub data: *mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}

impl<T> Mutex<T> {
    #[inline(always)]
    pub const fn new(data: T, name: &'static str) -> Self {
        Mutex {
            lock: AtomicBool::new(false),
            name,
            cpu_id: Cell::new(-1),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    #[inline(always)]
    pub fn lock(&self) -> MutexGuard<T> {
        push_off();
        if self.holding() {
            panic!("mutex {} acquire.", self.name);
        }
        loop {
            if let Some(guard) = self.try_lock_weak() {
                break guard;
            }

            while self.holding() {
                core::hint::spin_loop();
            }
        }
    }

    #[inline(always)]
    pub fn holding(&self) -> bool {
        self.is_locked() && self.cpu_id.get() == unsafe { cpu::cpuid() as isize }
    }

    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(core::sync::atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        if !self.holding() {
            panic!("Spinmutex {} release", self.name);
        }
        self.cpu_id.set(-1);
        fence(Ordering::SeqCst);
        self.lock.store(false, Ordering::Release);
        push_off();
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        push_off();
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            fence(Ordering::SeqCst);
            self.cpu_id.set(unsafe { cpu::cpuid() as isize });
            Some(MutexGuard {
                lock: self,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            pop_off();
            None
        }
    }

    #[inline(always)]
    pub fn try_lock_weak(&self) -> Option<MutexGuard<T>> {
        if self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            fence(Ordering::SeqCst);
            self.cpu_id.set(unsafe { cpu::cpuid() as isize });
            Some(MutexGuard {
                lock: self,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }

    #[inline(always)]
    pub unsafe fn raw_data_mut_unchecked(&self) -> *mut T {
        unsafe { self.data.as_mut_unchecked() }
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "Mutex {{ data: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, " }}")),
            None => write!(f, "Mutex {{ <locked> }}"),
        }
    }
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    #[inline(always)]
    pub fn leak(this: Self) -> &'a mut T {
        let mut this = ManuallyDrop::new(this);
        unsafe { &mut *this.data }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.lock.force_unlock() };
    }
}
