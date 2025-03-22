//! 页内存分配器

use core::{alloc::GlobalAlloc, ptr::null_mut};

use crate::{
    arch::riscv::qemu::layout::{KERNEL_BASE, PGSIZE, PHYSTOP},
    asm::end,
    lock::Mutex,
    memory::mapping::page_round_up,
    println,
};

#[global_allocator]
pub static SIMPLE_ALLOCATOR: LockedAllocator =
    LockedAllocator(Mutex::new(LinkedListAllocator::new(), "Simple allocator"));

pub fn init() {
    SIMPLE_ALLOCATOR.0.lock().init();
}

pub struct LockedAllocator(Mutex<LinkedListAllocator>);

struct LinkedListAllocator {
    head: Option<&'static mut Run>,
    count: [i32; (PHYSTOP - KERNEL_BASE) / PGSIZE],
}

struct Run {
    next: Option<&'static mut Run>,
}

impl LinkedListAllocator {
    const fn new() -> Self {
        Self {
            head: None,
            count: [0; (PHYSTOP - KERNEL_BASE) / PGSIZE],
        }
    }

    fn init(&mut self) {
        let heap_start = end as usize;
        let heap_start = page_round_up(heap_start);
        let heap_end = PHYSTOP;
        println!(
            "KernelHeap: available memory: [{:#x}, {:#x})",
            heap_start, PHYSTOP
        );
        self.head = None;
        for start in (heap_start..heap_end).step_by(PGSIZE) {
            // println!("{start:#x}");
            let node = unsafe { &mut *(start as *mut Run) };
            node.next = self.head.take();
            self.head = Some(node);
        }
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // assert!(layout.align(), 0x1000);
        assert!(layout.size() <= 0x1000);

        let mut g = self.0.lock();

        if let Some(node) = g.head.take() {
            let idx = (node as *mut _ as usize - KERNEL_BASE) / PGSIZE;
            g.count[idx] = 1;
            g.head = node.next.take();
            node as *mut Run as *mut u8
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // assert_eq!(layout.align(), 0x1000);
        // assert_eq!(layout.size(), 0x1000);
        assert!(layout.size() <= 0x1000);

        let mut g = self.0.lock();
        let head = ptr as *mut Run;
        let idx = (head as usize - KERNEL_BASE) / PGSIZE;
        g.count[idx] -= 1;
        if g.count[idx] == 0 {
            unsafe {
                (*head).next = g.head.take();
                g.head = Some(&mut *head);
            }
        }
    }
}

impl LockedAllocator {
    pub fn add_count(&self, pa: usize) {
        let idx = (pa - KERNEL_BASE) / PGSIZE;
        self.0.lock().count[idx] += 1;
    }

    pub fn get_count(&self, pa: usize) -> i32 {
        let idx = (pa - KERNEL_BASE) / PGSIZE;
        self.0.lock().count[idx]
    }
}
