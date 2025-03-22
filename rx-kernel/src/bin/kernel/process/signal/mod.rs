use core::cell::UnsafeCell;

use array_macro::array;
use bitflags::bitflags;

use crate::{arch::riscv::qemu::param::NPROC, lock::Mutex};

use super::{cpu::CPUManager, manager::PROC_MANAGER};

pub const NSIG: usize = 1;

pub const SIG_CHLD: usize = 0;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Sigset:usize {
        const Chld = 1 << SIG_CHLD;
    }
}

#[derive(Copy, Clone)]
pub enum SigHandler {
    Ignore,
    Default,
    // user handlers address
    // set epc jump here
    Handler(usize),
}

// signal可能有多种状态：未决、递送、处理

// 信号可能由多个进程同时修改，所以用一个互斥锁保护所有进程的信号集
// pending set则只会在进程运行时自己主动修改，所以在单线程的条件西无需保护
pub struct Signals {
    sigsets_received: Mutex<[Sigset; NPROC]>,
    // handlers runs by process itself, so it don't need be protected by lock
    handlers: UnsafeCell<[[SigHandler; NSIG]; NPROC]>,
    // pending sets
}

unsafe impl Sync for Signals {}

pub static SIGNALS: Signals = Signals::new();

impl Signals {
    pub const fn new() -> Self {
        Self {
            sigsets_received: Mutex::new([Sigset::empty(); NPROC], "sigsets"),
            handlers: UnsafeCell::new(array![_ => [SigHandler::Ignore; NSIG]; NPROC]),
        }
    }

    /// idx 是用于索引进程的PCB的索引号
    pub fn register(signum: usize, idx: usize, handler: isize) {
        unsafe {
            let handlers = SIGNALS.handlers.as_mut_unchecked();
            handlers[idx][signum] = if handler == 0 {
                SigHandler::Default
            } else if handler < 0 {
                SigHandler::Ignore
            } else {
                SigHandler::Handler(handler as usize)
            };
        }
    }

    /// 睡眠直到信号发生
    pub fn pause() {
        if let Some(p) = unsafe { CPUManager::myproc() } {
            let g = SIGNALS.sigsets_received.lock();
            let chan = &g[unsafe { p.data.as_ref_unchecked() }.id] as *const _ as usize;
            p.sleep(chan, g);

            // 信号发生时设置sigset并配置一份转到信号处理函数的trapframe，只处理一个信号
        }
    }

    /// 向进程发送信号
    pub fn kill(signum: usize, pid: usize) -> Result<(), &'static str> {
        if let Some(&id) = PROC_MANAGER.idx_pids.lock().iter().find(|p| **p == pid) {
            // 发送面对的进程有可能处于：
            //  1. 正在其它的CPU上运行，处理系统调用或者收到时钟中断又会回到内核态
            //  2. 在等待某个事件，处于睡眠的状态，当阻塞的事件发生时会在内核态中被唤醒（处理系统调用的过程中阻塞）
            //  2-1. 2的特例，在pause调用中阻塞，等待信号的发生

            // 如果该进程在等待信号则会被唤醒
            let mut g = SIGNALS.sigsets_received.lock();
            g[id].set(Sigset::from_bits_truncate(1 << signum), true);
            let chan = &g[id] as *const _ as usize;
            PROC_MANAGER.wake_up(chan);
            Ok(())
        } else {
            Err("No Such Process")
        }
    }
}
