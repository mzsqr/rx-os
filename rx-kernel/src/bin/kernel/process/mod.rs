//! 进程管理模块
//! 每个进程都包括运行在内核空间的部分以及运行在用户空间的部分
//! 当发生中断时进程会转入内核空间运行，这时：
//!     通过trapframe中保存的数据恢复CPU的内核现场
//!     1. 若由于时钟中断进入内核空间
//!        可能会将当前进程的内核上下文保存起来，并用CPU上保存的上一个上下文恢复内核现场，
//!        从而实现内核线程的切换，这一切换会回到上一个运行到的代码位置，通常是调度器上的
//!         switch调用。
//!     2. 若由于设备中断进入内核空间
//!         这时不会引发进程的切换，而是运行内核中的设备代码，将准备好的数据放好，并将等待
//!         设备中断的进程唤醒，然后又返回用户空间。
//!     3. 若由于系统调用进入内核空间
//!         这时根据约定的ABI获取用户空间提供的数据，进行相应的可能会使用户进程进入阻塞状态
//! 在内核运行的过程中当前进程对应的CPU上下文是动态变化的，在通过switch切换时将动态变化的
//! 上下文保存到进程的Contex上，并恢复内核在没有进程运行时的上下文（即调度器保存的位置），
//! 透过调度器又会将进程的内核上下文恢复到CPU上。
//!
//! 核心函数是`scheduler`, `sched`, `switch`
//! `scheduler`总是在内核没有进程运行的时候运行，所以最终总是将可调度的进程的结构中的Contex调度
//! 上CPU，`sched`则总是在内核有进程的时候运行，总是保存当前位置上下文，恢复没有进程运行时的上下文，
//! `switch`则是切换上下文的唯一方法。

use cpu::CPUManager;
use manager::PROC_MANAGER;
use process::ProcState;

use crate::{
    arch::riscv::qemu::fs::ROOTDEV,
    fs::{self, log::Log},
    trap::user_trap_ret,
};

pub mod context;
pub mod cpu;
pub mod elf;
pub mod manager;
pub mod process;
pub mod trapframe;

static INITCODE: &[u8] = &[
    0x17, 0x05, 0x00, 0x00, 0x13, 0x05, 0x45, 0x02, 0x97, 0x05, 0x00, 0x00, 0x93, 0x85, 0x35, 0x02,
    0x93, 0x08, 0x70, 0x00, 0x73, 0x00, 0x00, 0x00, 0x93, 0x08, 0x20, 0x00, 0x73, 0x00, 0x00, 0x00,
    0xef, 0xf0, 0x9f, 0xff, 0x2f, 0x69, 0x6e, 0x69, 0x74, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

unsafe fn fork_ret() {
    static mut FIRST: bool = true;

    // 父进程执行过程中透过调度器必然会持有meta锁，所以这里要强制解锁
    // 这和父进程是没有关联的
    unsafe {
        CPUManager::myproc().unwrap().meta.force_unlock();

        if FIRST {
            fs::init(ROOTDEV);
            FIRST = false;
        }
        user_trap_ret()
    };
}

pub fn exit(status: i32) -> ! {
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
    let mut wg = PROC_MANAGER.wait_list.lock();
    // TODO: reparent
    unsafe { PROC_MANAGER.reparent(&mut *wg, myproc) };
    PROC_MANAGER.wake_up(wg[pdata.id]);

    let mut pmeta = myproc.meta.lock();
    pmeta.xstate = status as usize;
    pmeta.set_state(ProcState::Zombie);
    drop(wg);
    drop(pmeta);

    unsafe { CPUManager::scheduler() };

    panic!("zombine exit!");
}
