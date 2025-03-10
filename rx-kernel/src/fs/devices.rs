use core::mem::transmute;

use crate::arch::riscv::qemu::param::NDEV;

type ReadFn = fn(bool, usize, usize) -> Option<usize>;
type WriteFn = fn(bool, usize, usize) -> Option<usize>;

pub struct DeviceList {
    pub table: [Device; NDEV],
}

impl DeviceList {
    const fn uinit() -> Self {
        Self {
            table: [Device::new(); NDEV],
        }
    }
}

#[derive(Copy, Clone)]
pub struct Device {
    pub read: *const u8,
    pub write: *const u8,
}

impl Device {
    const fn new() -> Self {
        Self {
            read: 0 as *const u8,
            write: 0 as *const u8,
        }
    }

    pub fn read(&self) -> ReadFn {
        unsafe { transmute::<*const u8, ReadFn>(self.read) }
    }

    pub fn write(&self) -> WriteFn {
        unsafe { transmute::<*const u8, WriteFn>(self.write) }
    }
}
