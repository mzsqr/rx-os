use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::process::{cpu::CPUManager, manager::PROC_MANAGER};

use super::Mutex;

pub struct SleepMutex<T: ?Sized> {
    locked: Mutex<bool>,
    name: &'static str,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SleepMutex<T> {}

impl<T> SleepMutex<T> {
    #[inline(always)]
    pub const fn new(data: T, name: &'static str) -> Self {
        Self {
            locked: Mutex::new(false, "Sleep lock"),
            name,
            data: UnsafeCell::new(data),
        }
    }
}

pub struct SleepMutexGuard<'a, T: ?Sized + 'a> {
    data: *mut T,
    guard: &'a SleepMutex<T>,
}

unsafe impl<T: ?Sized + Send> Send for SleepMutexGuard<'_, T> {}

impl<T: ?Sized> SleepMutex<T> {
    pub fn lock(&self) -> SleepMutexGuard<'_, T> {
        let mut guard = self.locked.lock();
        while *guard {
            unsafe {
                // sleep and release guard
                if let Some(p) = CPUManager::myproc() {
                    p.sleep(guard.data as usize, guard);
                }
            }
            guard = self.locked.lock();
        }
        *guard = true;
        drop(guard);
        SleepMutexGuard {
            guard: self,
            data: self.data.get(),
        }
    }

    pub fn unlock(&self) {
        let mut guard = self.locked.lock();
        *guard = false;
        self.wake_up(guard.data as usize);
        drop(guard);
    }

    fn wake_up(&self, chan: usize) {
        PROC_MANAGER.wake_up(chan);
    }
}

impl<T: ?Sized> Deref for SleepMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> DerefMut for SleepMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> Drop for SleepMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.guard.unlock();
    }
}
