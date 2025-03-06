//! 空闲物理空间的管理
//! 内核启动时需要对内存上空闲的物理空间进行管理，也就是调用本模块的初始化函数
//! 随后即可用Rust提供的Box指针分配内存
//! 也就是将这些空闲内存视为堆内存
//! 这些空闲空间主要是给进程分配页面的，所以在实际分配时要按照页面的抽象进行分配
//! 以页面对齐并且要分配页面大小的整数倍
//! TODO: 使用伙伴内存分配器

use core::alloc::Layout;

use linked_list_allocator::LockedHeap;

use crate::arch::riscv::qemu::layout::PHYSTOP;

use super::mapping::page_round_up;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    fn end();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("alloc error: {:?}", layout);
}

/// # Safety
/// 要在页表分配前对所有的空闲物理内存进行管理
pub unsafe fn init() {
    let heap_start = end as usize;
    let heap_start = page_round_up(heap_start);
    let heap_end = PHYSTOP;
    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start as *mut u8, heap_end - heap_start);
    }
}
