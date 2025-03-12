//! 分发设备读写函数
//! 要求设备自己实现同步和互斥

use core::{cell::UnsafeCell, mem::transmute, ptr::NonNull};

use crate::arch::riscv::qemu::param::NDEV;

type ReadFn = fn(bool, usize, usize) -> Option<usize>;
type WriteFn = fn(bool, usize, usize) -> Option<usize>;

pub static DEVICE_LIST: DeviceList = DeviceList::uninit();

unsafe impl Sync for DeviceList {}

pub struct DeviceList {
    pub table: UnsafeCell<[Device; NDEV]>,
}

impl DeviceList {
    const fn uninit() -> Self {
        Self {
            table: UnsafeCell::new([Device::new(); NDEV]),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Device {
    pub read: Option<ReadFn>,
    pub write: Option<WriteFn>,
}

impl Device {
    const fn new() -> Self {
        Self {
            read: None,
            write: None,
        }
    }

    pub fn read(&self) -> ReadFn {
        self.read.unwrap()
    }

    pub fn write(&self) -> WriteFn {
        self.write.unwrap()
    }
}
