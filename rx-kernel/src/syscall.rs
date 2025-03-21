use core::arch::asm;

use num_enum::{FromPrimitive, IntoPrimitive};

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

pub fn write(fd: i32, data: &[u8]) {
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
            in(reg) fd,
            in(reg) ptr,
            in(reg) l ,
            in(reg) sysn
        )
    }
}

pub fn exit(code: i32) {
    let sysn: usize = SyscallNum::SysExit.into();
    unsafe {
        asm!("mv a7, {}",
        "ecall",
        in(reg) sysn)
    }
}
