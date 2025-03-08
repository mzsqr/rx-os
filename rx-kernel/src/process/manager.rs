use core::sync::atomic::AtomicUsize;

use array_macro::array;
use spin::Mutex;

use crate::{
    arch::riscv::qemu::{
        layout::{PGSIZE, TRAMPOLINE},
        param::NPROC,
    },
    memory::{
        PageAllocator, Stack,
        address::{PhysicalAddress, VirtualAddress},
        mapping::{kernel_map::KERNEL_PAGETABLE, pagetable_entry::PteFlags},
    },
    println,
};

use super::process::Process;

// These data should use interior mutability
pub struct ProcManager {
    proc: [Process; NPROC],
    pids: AtomicUsize,
    // init proc,
    // 每个进程的父进程
    pub wait_list: Mutex<[usize; NPROC]>,
}

pub static PROC_MANAGER: ProcManager = ProcManager::new();

impl ProcManager {
    pub const fn new() -> Self {
        Self {
            proc: array![id => Process::new(id);NPROC],
            pids: AtomicUsize::new(0),
            wait_list: Mutex::new([0; NPROC]),
        }
    }

    pub fn alloc_pid(&self) -> usize {
        self.pids.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }

    /// 初始化进程表
    ///
    /// # Safety
    /// 仅在启动时调用
    pub unsafe fn init(&self) {
        println!("process init...");
        for (pos, proc) in self.proc.iter().enumerate() {
            // kernel stack is not allocated
            proc.init(kernel_stack(pos));
        }
    }

    /// 每个进程分配一个内核栈
    /// 将其映射到内核页表的高地址段
    /// 还包括一个保护页（非valid）
    ///
    /// # Safety
    /// 初始化内核页表后调用
    pub unsafe fn proc_mapstacks(&self) {
        for i in 0..self.proc.len() {
            let pa = unsafe { Stack::new_zeroed() };
            let pa = pa as *mut Stack as usize;
            let va = kernel_stack(i);

            unsafe {
                KERNEL_PAGETABLE.pgt.as_mut_unchecked().kernel_map(
                    VirtualAddress::new(va),
                    PhysicalAddress::new(pa),
                    PGSIZE * 4,
                    PteFlags::R | PteFlags::W,
                )
            };
        }
    }
}

pub unsafe fn init() {
    unsafe { PROC_MANAGER.init() };
}

#[inline]
fn kernel_stack(pos: usize) -> usize {
    TRAMPOLINE - (pos + 1) * 5 * PGSIZE
}
