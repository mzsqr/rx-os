//! `rx-os`的内存管理模块
//! 本模块的主要内容有：
//!     物理地址和虚拟地址的抽象
//!     空闲物理地址的管理
//!     页面/页表/栈的抽象
//!     虚拟地址-物理地址之间的映射

use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use alloc::boxed::Box;

use crate::{
    arch::riscv::qemu::layout::{PGSIZE, STACK_SIZE},
    process::{cpu::CPUManager, manager::PROC_MANAGER},
};

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
    data: [u8; STACK_SIZE],
}

impl PageAllocator for Stack {}

/// 从用户空间或内核空间中拷贝数据
pub fn copy_to_kernel(
    dst: &mut [u8],
    src: usize,
    is_user: bool,
    count: usize,
) -> Result<(), &'static str> {
    if is_user {
        let myproc = unsafe { CPUManager::myproc() }.unwrap();
        let pgt = unsafe { myproc.data.as_mut_unchecked() }
            .pagetable
            .as_deref_mut()
            .unwrap();
        let _ = pgt.copy_in(dst, src);
    } else {
        let src = unsafe { &*slice_from_raw_parts(src as *mut u8, count) };
        dst.copy_from_slice(src);
    }
    Ok(())
}

/// 将内核数据复制到其它位置
pub fn copy_from_kernel(
    dst: usize,
    src: &[u8],
    is_user: bool,
    count: usize,
) -> Result<(), &'static str> {
    if is_user {
        let myproc = unsafe { CPUManager::myproc() }.unwrap();
        let pgt = unsafe { myproc.data.as_mut_unchecked() }
            .pagetable
            .as_deref_mut()
            .unwrap();
        pgt.copy_out(dst, src);
    } else {
        let dst = unsafe { &mut *slice_from_raw_parts_mut(dst as *mut u8, count) };
        dst.copy_from_slice(src);
    }
    Ok(())
}

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
