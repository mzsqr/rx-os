//! `rx-os`的内存管理模块
//! 本模块的主要内容有：
//!     物理地址和虚拟地址的抽象
//!     空闲物理地址的管理
//!     页面/页表/栈的抽象
//!     虚拟地址-物理地址之间的映射

use alloc::boxed::Box;

use crate::arch::riscv::qemu::layout::PGSIZE;

pub mod address;
pub mod kalloc;
pub mod mapping;

pub trait PageAllocator: Sized {
    unsafe fn new_zeroed() -> &'static mut Self {
        unsafe {
            let page = Box::<Self>::new_zeroed().assume_init();
            Box::leak(page)
        }
    }

    unsafe fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self as *mut Self) };
    }
}

#[repr(C, align(4096))]
pub struct RawPage {
    data: [u8; PGSIZE],
}

impl PageAllocator for RawPage {}

#[repr(C, align(4096))]
pub struct Stack {
    data: [u8; PGSIZE * 4],
}

impl PageAllocator for Stack {}

#[cfg(test)]
mod test {
    use core::ptr::drop_in_place;

    use alloc::boxed::Box;

    use crate::{arch::riscv::qemu::layout::PGSIZE, println};

    use super::{PageAllocator, RawPage, kalloc::ALLOCATOR};

    #[test_case]
    fn test_drop_in_place() {
        let prev = ALLOCATOR.lock().used() as usize;
        let pg = unsafe { RawPage::new_zeroed() };
        let mid = ALLOCATOR.lock().used() as usize;
        unsafe { pg.drop() };
        let after = ALLOCATOR.lock().used() as usize;

        assert_eq!(prev, after, "Not Allocated or Not Deallocated");
        assert_eq!(prev + PGSIZE, mid, "Not Allocated");
    }
}
