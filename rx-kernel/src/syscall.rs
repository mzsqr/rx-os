use core::arch::asm;

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::fs::Stat;

#[derive(Debug, FromPrimitive, IntoPrimitive)]
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

fn syscall_res() -> isize {
    let x;
    unsafe { asm!("mv {}, a0", out(reg) x) };
    x
}

pub fn fork() -> isize {
    let sysn: usize = SyscallNum::SysFork.into();
    unsafe {
        asm!(
            "mv a7, {}",
            "ecall",
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn wait(status: &mut isize) -> isize {
    let sysn: usize = SyscallNum::SysWait.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) status as *mut isize as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn pipe(fds: &mut [i32]) -> isize {
    if fds.len() < 2 {
        return -1;
    }
    let sysn: usize = SyscallNum::SysPipe.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fds.as_mut_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// must be zero-ended
pub fn exec(path: &[u8], argv: &[*const u8]) -> isize {
    // TODO: error rectify
    if path[path.len() - 1] != 0 || !argv[argv.len() - 1].is_null() {
        return -1;
    }
    let sysn: usize = SyscallNum::SysExec.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) argv.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn fstat(fd: i32, stat: &mut Stat) -> isize {
    let sysn: usize = SyscallNum::SysFstat.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fd as usize,
            in(reg) stat as *mut Stat as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn chdir(path: &[u8]) -> isize {
    let sysn: usize = SyscallNum::SysChdir.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn dup(fd: i32) -> isize {
    let sysn: usize = SyscallNum::SysDup.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fd as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn getpid() -> isize {
    let sysn: usize = SyscallNum::SysGetPid.into();
    unsafe {
        asm!(
            "mv a7, {}",
            "ecall",
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn sbrk(size: usize) -> isize {
    let sysn: usize = SyscallNum::SysSbrk.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) size,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn sleep(num: usize) -> isize {
    let sysn: usize = SyscallNum::SysSleep.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) num,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn uptime() -> isize {
    let sysn: usize = SyscallNum::SysUptime.into();
    unsafe {
        asm!(
            "mv a7, {}",
            "ecall",
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn open(path: &[u8]) -> isize {
    let sysn: usize = SyscallNum::SysOpen.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn mknod(path: &[u8], major: i16, minor: i16) -> isize {
    let sysn: usize = SyscallNum::SysMknod.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a2, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) major as usize,
            in(reg) minor as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn close(fd: i32) -> isize {
    let sysn: usize = SyscallNum::SysClose.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fd as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn unlink(path: &[u8]) -> isize {
    let sysn: usize = SyscallNum::SysUnlink.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn link(old_path: &[u8], new_path: &[u8]) -> isize {
    let sysn: usize = SyscallNum::SysLink.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a7, {}",
            "ecall",
            in(reg) old_path.as_ptr() as usize,
            in(reg) new_path.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

/// zero-ended
pub fn mkdir(path: &[u8]) -> isize {
    let sysn: usize = SyscallNum::SysMkdir.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) path.as_ptr() as usize,
            in(reg) sysn
        );
    }

    syscall_res()
}

pub fn read(fd: i32, data: &mut [u8]) -> isize {
    let ptr = data.as_mut_ptr();
    let l = data.len();
    let sysn: usize = SyscallNum::SysWrite.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a2, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fd as usize,
            in(reg) ptr,
            in(reg) l ,
            in(reg) sysn
        )
    }
    syscall_res()
}

pub fn write(fd: i32, data: &[u8]) -> isize {
    let ptr = data.as_ptr();
    let l = data.len();
    let sysn: usize = SyscallNum::SysWrite.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a2, {}",
            "mv a7, {}",
            "ecall",
            in(reg) fd as usize,
            in(reg) ptr,
            in(reg) l ,
            in(reg) sysn
        )
    }
    syscall_res()
}

pub fn exit(code: i32) {
    let sysn: usize = SyscallNum::SysExit.into();
    unsafe {
        asm!(
            "mv a0, {}",
            "mv a7, {}",
            "ecall",
            in(reg) code,
            in(reg) sysn
        )
    }
}
