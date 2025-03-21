//! 处理系统调用核心功能的入口，由trap模块调用。
//! 用户空间通过ecall进入内核时：
//!     将调用约定的参数放在a0-a5寄存器中，将系统调用号放在a7寄存器中。
//!     在跳板代码中将这些参数保存到trapframe中。
//!     本模块通过这些参数信息再调用系统提供的具体功能。

use num_enum::FromPrimitive;
use rx_kernel::syscall::SyscallNum;

use crate::{
    println,
    process::{cpu::CPUManager, process::Process},
};

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
    let syscall = Syscall { process: proc };
    let ret = if let Ok(res) = syscall.syscall() {
        res
    } else {
        -1_isize as usize
    };
    unsafe {
        let tf = proc.data.as_ref_unchecked().trapframe;
        (*tf).ax[0] = ret;
    }
    // save return value in a0
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
            SyscallNum::SysFork => self.sys_fork(),
            SyscallNum::SysExit => self.sys_exit(),
            SyscallNum::SysWait => self.sys_wait(),
            SyscallNum::SysPipe => self.sys_pipe(),
            SyscallNum::SysRead => self.sys_read(),
            SyscallNum::SysKill => unimplemented!(),
            SyscallNum::SysExec => self.sys_exec(),
            SyscallNum::SysFstat => self.sys_fstat(),
            SyscallNum::SysChdir => self.sys_chdir(),
            SyscallNum::SysDup => self.sys_dup(),
            SyscallNum::SysGetPid => self.sys_getpid(),
            SyscallNum::SysSbrk => self.sys_sbrk(),
            SyscallNum::SysSleep => self.sys_sleep(),
            SyscallNum::SysUptime => Ok(0),
            SyscallNum::SysOpen => self.sys_open(),
            SyscallNum::SysWrite => self.sys_write(),
            SyscallNum::SysMknod => self.sys_mknod(),
            SyscallNum::SysUnlink => self.sys_unlink(),
            SyscallNum::SysLink => self.sys_link(),
            SyscallNum::SysMkdir => self.sys_mkdir(),
            SyscallNum::SysClose => self.sys_close(),
            SyscallNum::Unknown => unimplemented!(),
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
            println!("[Debug] addr: 0x{:x}", addr);
            println!("[Debug] pdata size: 0x{:x}", pdata.size);
            panic!("Invalid  user virtual address");
        }

        let pgt = pdata.pagetable.as_deref_mut().unwrap();
        pgt.copy_in(buf, addr)?;
        Ok(())
    }
}
