use core::{
    cell::{Cell, UnsafeCell},
    sync::atomic::{AtomicBool, AtomicUsize},
};

pub struct RwLock<T: ?Sized> {
    lock: AtomicUsize,
    name: &'static str,
    cpu_id: Cell<isize>,
    data: UnsafeCell<T>,
}

/// A guard that provides shared read access.
///
/// When the guard falls out of scope it will release the lock.
pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
    pub data: *mut T,
}

/// A guard that provides mutex write access.
///
/// When the guard falls out of scope it will release the lock.
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
    pub data: *mut T,
}

/// A guard that provides upgradable read access.
///
/// When the guard falls out of scope it will release the lock.
pub struct RwLockUpgradableGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
    pub data: *mut T,
}

// Same unsafe impls as `std::sync::RwLock`
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

unsafe impl<T: ?Sized + Send + Sync> Send for RwLockWriteGuard<'_, T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLockWriteGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Send for RwLockReadGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}

unsafe impl<T: ?Sized + Send + Sync> Send for RwLockUpgradableGuard<'_, T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLockUpgradableGuard<'_, T> {}
