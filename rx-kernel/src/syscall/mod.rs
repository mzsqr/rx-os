use num_enum::FromPrimitive;

use crate::process::{cpu::CPUManager, process::Process};

mod file;
mod proc;

#[inline]
pub fn kernel_env_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!("ecall"
            , inlateout("x10") arg0 => ret,
             in("x11") arg1, in("x12") arg2, in("x17") which,
        );
    }
    ret
}

pub fn syscall_handler() {
    let proc = unsafe { CPUManager::myproc().unwrap() };

    // call syscall
    // save return value in a0
}

#[derive(Debug, FromPrimitive)]
#[repr(usize)]
pub enum SyscallNum {
    SysFork = 1,
    SysExit = 2,
    SysWait = 3,
    SysPipe = 4,
    SysRead = 5,
    SysKill = 6,
    SysExec = 7,
    SysFstat = 8,
    SysChdir = 9,
    SysDup = 10,
    SysGetPid = 11,
    SysSbrk = 12,
    SysSleep = 13,
    SysUptime = 14,
    SysOpen = 15,
    SysWrite = 16,
    SysMknod = 17,
    SysUnlink = 18,
    SysLink = 19,
    SysMkdir = 20,
    SysClose = 21,
    #[default]
    Unknown,
}

pub struct Syscall<'a> {
    process: &'a Process,
}

type SysResult = Result<usize, ()>;

impl Syscall<'_> {
    pub fn syscall(&self) -> SysResult {
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let syscall_num = unsafe { (*pdata.trapframe).ax[7] };
        let syscall_num = SyscallNum::from_primitive(syscall_num);
        match syscall_num {
            SyscallNum::SysFork => todo!(),
            SyscallNum::SysExit => todo!(),
            SyscallNum::SysWait => todo!(),
            SyscallNum::SysPipe => todo!(),
            SyscallNum::SysRead => todo!(),
            SyscallNum::SysKill => todo!(),
            SyscallNum::SysExec => todo!(),
            SyscallNum::SysFstat => todo!(),
            SyscallNum::SysChdir => todo!(),
            SyscallNum::SysDup => todo!(),
            SyscallNum::SysGetPid => todo!(),
            SyscallNum::SysSbrk => todo!(),
            SyscallNum::SysSleep => todo!(),
            SyscallNum::SysUptime => todo!(),
            SyscallNum::SysOpen => todo!(),
            SyscallNum::SysWrite => todo!(),
            SyscallNum::SysMknod => todo!(),
            SyscallNum::SysUnlink => todo!(),
            SyscallNum::SysLink => todo!(),
            SyscallNum::SysMkdir => todo!(),
            SyscallNum::SysClose => todo!(),
            SyscallNum::Unknown => todo!(),
        }
    }

    pub fn arg(&self, id: usize) -> usize {
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let tf = unsafe { &(*pdata.trapframe) };
        if id > 5 {
            panic!("too many argument.");
        }
        tf.ax[id]
    }

    pub fn copy_from_str(&self, addr: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let pgt = pdata.pagetable.as_deref_mut().unwrap();
        pgt.copy_in_str(buf, addr)?;
        Ok(())
    }

    pub fn copy_from_addr(&self, addr: usize, buf: &mut [u8]) -> Result<(), &'static str> {
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        if addr > pdata.size || addr + size_of::<usize>() > pdata.size {
            panic!("Invalid  user virtual address");
        }

        let pgt = pdata.pagetable.as_deref_mut().unwrap();
        pgt.copy_in(buf, addr)?;
        Ok(())
    }
}
