use core::cell::UnsafeCell;

use crate::{asm::switch, lock::MutexGuard};
use array_macro::array;

use crate::{
    arch::riscv::{
        qemu::param::NCPU,
        register::{sstatus, tp},
    },
    println,
    process::manager::PROC_MANAGER,
};

use super::{
    context::Context,
    process::{ProcMeta, ProcState, Process},
};

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub process: Option<&'static Process>, // The process running on this cpu, or null.
    pub context: Context,                  // swtch() here to enter scheduler().
    pub noff: usize,                       // Depth of push_off() nesting.
    pub intena: bool,                      // Were interrupts enabled before push_off()?
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

    pub fn set_proc(&mut self, proc: Option<&'static Process>) {
        self.process = proc;
    }

    pub fn get_context_mut(&mut self) -> *mut Context {
        &raw mut self.context
    }

    /// Switch to scheduler.  Must hold only p->lock
    /// and have changed proc->state. Saves and restores
    /// intena because intena is a property of this
    /// kernel thread, not this CPU. It should
    /// be proc->intena and proc->noff, but that would
    /// break in the few places where a lock is held but
    /// there's no process.
    pub unsafe fn sched<'a>(
        &mut self,
        guard: MutexGuard<'a, ProcMeta>,
        ctx: *mut Context,
    ) -> MutexGuard<'a, ProcMeta> {
        if self.noff != 1 {
            println!("self noff is {}", self.noff);
            panic!("sched: cpu hold multiple locks");
        }

        if let ProcState::Running = guard.state {
            panic!("sched: proc is running");
        }

        if unsafe { sstatus::intr_get() } {
            panic!("sched: interruptible");
        }

        let intena = self.intena;
        unsafe { switch(ctx, self.get_context_mut()) };

        self.intena = intena;
        guard
    }

    pub fn try_yield_proc(&mut self) {
        if let Some(p) = self.process {
            let g = p.meta.lock();
            if let ProcState::Running = g.state {
                drop(g);
                p.yielding();
            } else {
                drop(g);
            }
        }
    }
}

/// internal data struct
pub struct CPUManager {
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
    pub unsafe fn mycpu() -> &'static mut CPU {
        unsafe { &mut CPU_MANAGER.cpus.as_mut_unchecked()[cpuid()] }
    }

    pub unsafe fn myproc() -> Option<&'static Process> {
        push_off();
        let c = unsafe { Self::mycpu() };
        // in this immutable ref we can copy it
        let p = c.process;
        pop_off();
        p
    }

    pub fn yield_proc() {
        if let Some(my_proc) = unsafe { Self::myproc() } {
            let st = my_proc.meta.lock().state;
            if let ProcState::Running = st {
                drop(st);
                my_proc.yielding();
            } else {
                drop(st);
            }
        }
    }

    pub unsafe fn scheduler() {
        let c = unsafe { Self::mycpu() };
        loop {
            unsafe { sstatus::intr_on() };

            // use seek runnable is not fair
            // for p in &PROC_MANAGER.proc {
            //     // if unsafe { cpuid() } == 0 {
            //     //     println!("scheduler {}", unsafe { p.data.as_ref_unchecked().id });
            //     // }
            //     // let mut g = p.meta.lock();
            //     if let Some(mut g) = p.meta.try_lock() {
            //         // println!("check {}", unsafe { p.data.as_ref_unchecked().id });
            //         if let ProcState::Runnable = g.state {
            //             c.set_proc(Some(p));
            //             g.state = ProcState::Running;
            //             unsafe {
            //                 switch(
            //                     c.get_context_mut(),
            //                     &mut p.data.as_mut_unchecked().context as *mut _,
            //                 );
            //             }
            //             c.set_proc(None);
            //         }
            //         drop(g);
            //     }
            // }

            if let Some(p) = PROC_MANAGER.seek_runnable() {
                c.set_proc(Some(p));
                let mut pmeta = p.meta.lock();
                pmeta.state = ProcState::Running;
                unsafe {
                    switch(
                        c.get_context_mut(),
                        p.data.as_mut_unchecked().get_context_mut(),
                    )
                };
                c.set_proc(None);
                drop(pmeta);
            }
        }
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
    let my_cpu = unsafe { CPUManager::mycpu() };
    if my_cpu.noff == 0 {
        my_cpu.intena = old_enable;
    }

    my_cpu.noff += 1;
}

pub fn pop_off() {
    if unsafe { sstatus::intr_get() } {
        panic!("pop_off(): interruptable");
    }
    let c = unsafe { CPUManager::mycpu() };
    if c.noff.checked_sub(1).is_none() {
        panic!("pop_off(): count not match");
    }
    c.noff -= 1;
    if c.noff == 0 && c.intena {
        unsafe { sstatus::intr_on() };
    }
}
