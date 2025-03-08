use core::cell::UnsafeCell;

use array_macro::array;

use crate::arch::riscv::{
    qemu::param::NCPU,
    register::{sstatus, tp},
};

use super::context::Context;

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub process: Option<usize>, // The process running on this cpu, or null.
    pub context: Context,       // swtch() here to enter scheduler().
    pub noff: usize,            // Depth of push_off() nesting.
    pub intena: bool,           // Were interrupts enabled before push_off()?
}

impl CPU {
    pub const fn new() -> Self {
        Self {
            process: None,
            context: Context::new(),
            noff: 0,
            intena: false,
        }
    }

    pub fn set_proc(&mut self, proc: Option<usize>) {
        self.process = proc;
    }

    pub fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

/// internal data struct
struct CPUManager {
    cpus: UnsafeCell<[CPU; NCPU]>,
}

/// # Safety
/// 每个CPU都将自己的id保存在tp寄存器中
/// 所以每个CPU都不会修改其它CPU上的内容
/// 因此不需要锁进行互斥的访问
unsafe impl Sync for CPUManager {}

static CPU_MANAGER: CPUManager = CPUManager::new();

impl CPUManager {
    pub const fn new() -> Self {
        Self {
            cpus: UnsafeCell::new(array![_ => CPU::new(); NCPU]),
        }
    }

    /// 获取当前CPU
    pub fn mycpu() -> &'static mut CPU {
        unsafe { &mut CPU_MANAGER.cpus.as_mut_unchecked()[cpuid()] }
    }
}

/// # Safety
/// this function should be called after save hartid in tp(per hart local register)
pub unsafe fn cpuid() -> usize {
    unsafe { tp::read() }
}

/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
pub fn push_off() {
    let old_enable;
    unsafe {
        old_enable = sstatus::intr_get();
        sstatus::intr_off();
    }
    let my_cpu = CPUManager::mycpu();
    if my_cpu.noff == 0 {
        my_cpu.intena = old_enable;
    }

    my_cpu.noff += 1;
}

pub fn pop_off() {
    if unsafe { sstatus::intr_get() } {
        panic!("pop_off(): interruptable");
    }
    let c = CPUManager::mycpu();
    if c.noff.checked_sub(1).is_none() {
        panic!("pop_off(): count not match");
    }
    c.noff -= 1;
    if c.noff == 0 && c.intena {
        unsafe { sstatus::intr_on() };
    }
}
