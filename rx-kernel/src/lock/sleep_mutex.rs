//! sleeplock

use core::cell::{Cell, UnsafeCell};
use core::ops::{Deref, DerefMut, Drop};

use crate::process::cpu::CPUManager;
use crate::process::manager::PROC_MANAGER;

use super::Mutex;

pub struct SleepChannel(u8);

pub struct SleepMutex<T: ?Sized> {
    lock: Mutex<()>,
    locked: Cell<bool>,
    chan: SleepChannel,
    name: &'static str,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SleepMutex<T> {}
// not needed
// unsafe impl<T: ?Sized + Sync> Send for SleepLock<T> {}

impl<T> SleepMutex<T> {
    pub const fn new(data: T, name: &'static str) -> Self {
        Self {
            lock: Mutex::new((), "sleeplock"),
            locked: Cell::new(false),
            chan: SleepChannel(0),
            name,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SleepMutex<T> {
    /// non-blocking, but might sleep if other p lock this sleeplock
    pub fn lock(&self) -> SleepMutexGuard<T> {
        let mut guard = self.lock.lock();
        while self.locked.get() {
            unsafe {
                CPUManager::myproc()
                    .unwrap()
                    .sleep(self.locked.as_ptr() as usize, guard);
            }
            guard = self.lock.lock();
        }
        self.locked.set(true);
        drop(guard);
        SleepMutexGuard {
            lock: &self,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Called by its guard when dropped
    pub fn unlock(&self) {
        let guard = self.lock.lock();
        self.locked.set(false);
        self.wake_up();
        drop(guard);
    }

    fn wake_up(&self) {
        unsafe {
            PROC_MANAGER.wake_up(self.locked.as_ptr() as usize);
        }
    }
}

pub struct SleepMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a SleepMutex<T>,
    data: &'a mut T,
}

impl<'a, T: ?Sized> Deref for SleepMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SleepMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SleepMutexGuard<'a, T> {
    /// The dropping of the SpinLockGuard will call spinlock's release_lock(),
    /// through its reference to its original spinlock.
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
