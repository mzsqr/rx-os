use core::sync::atomic::Ordering;

use crate::{
    process::{
        cpu::CPUManager,
        manager::{PROC_MANAGER, ProcManager},
    },
    trap::TICKS,
};

use super::{SysResult, Syscall};

impl Syscall<'_> {
    pub fn sys_fork(&self) -> SysResult {
        // TODO: Determine this lock logic
        let child_proc = self.process.fork().ok_or(())?;
        let pmeta = child_proc.meta.lock();
        let pid = pmeta.pid;
        Ok(pid)
    }

    pub fn sys_exit(&self) -> SysResult {
        let status = self.arg(0);
        PROC_MANAGER.exit(status)
    }

    pub fn sys_wait(&self) -> SysResult {
        let addr = self.arg(0);
        PROC_MANAGER.wait(addr).ok_or(())
    }

    pub fn sys_getpid(&self) -> SysResult {
        Ok(self.process.pid())
    }

    pub fn sys_sbrk(&mut self) -> SysResult {
        let size = self.arg(0);
        let pdata = unsafe { self.process.data.as_ref_unchecked() };
        let addr = pdata.size;
        if let Err(err) = self.process.grow_proc(size as isize) {
            panic!("err: {:?}", err);
        } else {
            Ok(addr)
        }
    }

    pub fn sys_sleep(&self) -> SysResult {
        let time_span = self.arg(0);

        let mut tg = TICKS.lock();
        let now_time = *tg;
        while *tg - now_time < time_span {
            let proc = unsafe { CPUManager::myproc().expect("Failed to get my process") };
            if proc.killed() {
                return Err(());
            }
            proc.sleep(0, tg);
            tg = TICKS.lock();
        }
        drop(tg);
        Ok(0)
    }
}
