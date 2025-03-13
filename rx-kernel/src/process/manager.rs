use core::ptr::slice_from_raw_parts;
use core::sync::atomic::AtomicUsize;

use array_macro::array;
use spin::Once;

use crate::arch::riscv::qemu::fs::ROOTIPATH;
use crate::fs::inode::ICACHE;
use crate::fs::log::Log;
use crate::lock::Mutex;

use crate::process::context::Context;
use crate::process::cpu::CPUManager;
use crate::{
    arch::riscv::qemu::{
        layout::{PGSIZE, TRAMPOLINE},
        param::NPROC,
    },
    memory::{
        PageAllocator, RawPage, Stack,
        address::{PhysicalAddress, VirtualAddress},
        mapping::{kernel_map::KERNEL_PAGETABLE, pagetable_entry::PteFlags},
    },
    println,
    process::INITCODE,
};

use super::{
    process::{ProcState, Process},
    trapframe::Trapframe,
};

// These data should use interior mutability
pub struct ProcManager {
    pub proc: [Process; NPROC],
    pids: AtomicUsize,
    init_proc: Once<usize>,
    // 每个进程的父进程
    pub wait_list: Mutex<[usize; NPROC]>,
}

pub static PROC_MANAGER: ProcManager = ProcManager::new();

impl ProcManager {
    pub const fn new() -> Self {
        Self {
            proc: array![id => Process::new(id, "Proc mutex");NPROC],
            pids: AtomicUsize::new(0),
            init_proc: Once::new(),
            wait_list: Mutex::new([0; NPROC], "Wait List"),
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

    pub unsafe fn user_init(&self) {
        println!("first user process init......");

        if let Some(p) = self.alloc_proc() {
            let pdata = unsafe { p.data.as_mut_unchecked() };
            pdata.pagetable.as_mut().into_iter().for_each(|pgt| {
                unsafe { pgt.uinit(&INITCODE) };
            });
            pdata.size = PGSIZE;

            let tf = unsafe { &mut *pdata.trapframe };
            tf.epc = 0;
            tf.sp = 4 * PGSIZE;
            pdata.set_name("initprog\0".as_bytes());
            pdata.cwd = Some(ICACHE.namei(&ROOTIPATH).expect("cannot find root inode"));
            p.meta
                .lock()
                .set_state(crate::process::process::ProcState::Runnable);

            self.init_proc.call_once(|| p as *const _ as usize);
        } else {
            panic!("Failed to get unused process");
        }
    }

    pub fn alloc_proc(&self) -> Option<&Process> {
        let pid = self.alloc_pid();
        for proc in &self.proc {
            let mut g = proc.meta.lock();
            if let super::process::ProcState::Unused = g.state {
                g.pid = pid;
                g.set_state(super::process::ProcState::Allocated);
                let pdata = unsafe { proc.data.as_mut_unchecked() };
                let tf = unsafe { RawPage::new_zeroed() };
                // when you free it
                // you should use RawPage other than Trapframe
                pdata.set_trapframe(tf as *mut RawPage as *mut Trapframe);
                pdata.init_context();
                proc.proc_pagetable();
                return Some(proc);
            }
        }
        None
    }

    pub fn wake_up(&self, channel: usize) {
        for p in self.proc.iter() {
            let mut g = p.meta.lock();
            if let ProcState::Sleeping = g.state {
                if g.chan == channel {
                    g.state = ProcState::Runnable;
                }
            }
        }
    }

    pub fn seek_runnable(&self) -> Option<&Process> {
        for p in self.proc.iter() {
            let mut g = p.meta.lock();
            if let ProcState::Runnable = g.state {
                g.state = ProcState::Allocated;
                return Some(p);
            }
        }
        None
    }

    /// 要提前持有wait锁
    /// 将该进程的所有子进程的父进程改为init
    pub fn reparent(&self, wait_list: &mut [usize], proc: &Process) {
        let addr = proc as *const _ as usize;
        // wait_list 和 init_proc存的都是地址
        for p in wait_list.iter_mut() {
            if *p == addr {
                *p = *self.init_proc.get().unwrap();
                self.wake_up(*p);
            }
        }
    }

    pub fn exit(&self, status: usize) -> ! {
        let myproc = unsafe { CPUManager::myproc().expect("Curretn Process has no cpu") };
        let pdata = unsafe { myproc.data.as_mut_unchecked() };
        for fd in &mut pdata.open_files {
            let _ = fd.take();
        }

        Log::begin_op();
        let cwd = pdata.cwd.take();
        drop(cwd);
        Log::end_op();

        // 将子进程的父进程改为init
        // 通知父进程的等待
        let mut wg = self.wait_list.lock();
        self.reparent(&mut *wg, myproc);
        self.wake_up(wg[pdata.id]);

        let mut pmeta = myproc.meta.lock();
        pmeta.xstate = status;
        pmeta.set_state(ProcState::Zombie);
        drop(wg);

        let cpu = unsafe { CPUManager::mycpu() };
        unsafe { cpu.sched(pmeta, &mut pdata.context as *mut Context) };

        panic!("zombine exit!");
    }

    pub fn wait(&self, addr: usize) -> Option<usize> {
        let proc = unsafe { CPUManager::myproc().unwrap() };
        let mut wg = self.wait_list.lock();
        loop {
            let mut have_kids = false;
            let pid;
            for p in &self.proc {
                let pdata = unsafe { p.data.as_mut_unchecked() };
                let parent_addr = wg[pdata.id];
                if parent_addr == proc as *const _ as usize {
                    let pmeta = p.meta.lock();
                    have_kids = true;
                    if pmeta.state == ProcState::Zombie {
                        pid = pmeta.pid;
                        let pgt = pdata.pagetable.as_deref_mut().unwrap();
                        let src = unsafe {
                            &*slice_from_raw_parts(
                                &pmeta.xstate as *const _ as *const u8,
                                size_of::<usize>(),
                            )
                        };
                        if addr != 0 && pgt.copy_out(addr, src).is_err() {
                            return None;
                        }
                        drop(pmeta);
                        p.free_proc();
                        return Some(pid);
                    }
                }
            }
            let pmeta = proc.meta.lock();
            if !have_kids || pmeta.killed {
                return None;
            }
            drop(pmeta);
            proc.sleep(proc as *const _ as usize, wg);
            wg = self.wait_list.lock();
        }
    }

    pub fn kill(&self, pid: usize) -> Result<usize, ()> {
        for p in &self.proc {
            let mut pmeta = p.meta.lock();
            if pmeta.pid == pid {
                pmeta.killed = true;
                if pmeta.state == ProcState::Sleeping {
                    pmeta.state = ProcState::Runnable;
                    return Ok(0);
                }
            }
        }
        Err(())
    }

    /// 打印进程列表
    pub fn proc_dump(&self) {
        for p in &self.proc {
            let pmeta = p.meta.lock();
            if pmeta.state != ProcState::Unused {
                println!(
                    "pid: {} state {:?} name: {:?}",
                    pmeta.pid,
                    pmeta.state,
                    p.name()
                );
            }
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
