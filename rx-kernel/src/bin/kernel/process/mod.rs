use cpu::CPUManager;
use manager::PROC_MANAGER;
use process::ProcState;

use crate::{
    arch::riscv::qemu::fs::ROOTDEV,
    fs::{self, log::Log},
    println,
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
    let wg = PROC_MANAGER.wait_list.lock();
    // TODO: reparent
    // PROC_MANAGER.reparent(&mut *wg, myproc);
    PROC_MANAGER.wake_up(wg[pdata.id]);

    let mut pmeta = myproc.meta.lock();
    pmeta.xstate = status as usize;
    pmeta.set_state(ProcState::Zombie);
    drop(wg);
    drop(pmeta);

    unsafe { CPUManager::scheduler() };

    panic!("zombine exit!");
}
