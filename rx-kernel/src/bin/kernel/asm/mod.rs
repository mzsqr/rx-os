//! 系统中用汇编编写的函数定义

use core::arch::global_asm;

use crate::{
    arch::riscv::qemu::{layout::STACK_SIZE, param::NCPU},
    process::context::Context,
};

global_asm!(include_str!("entry.S"));
global_asm!(include_str!("kernelvec.S"));
global_asm!(include_str!("switch.S"));
global_asm!(include_str!("trampoline.S"));

// define in Assembly files.
unsafe extern "C" {
    pub fn kernelvec();
    pub fn timervec();
    pub fn switch(old_ctx: *mut Context, new_ctx: *const Context);
    pub fn trampoline();
    pub fn uservec();
    pub fn userret();
}

// define in linker script
// [u8;0]的定义也可用，但是采用函数的形式可以直接的转为地址
unsafe extern "C" {
    pub fn etext();
    pub fn end();
}

// 为什么非要把这个连接到数据段才行呢？
/// 初始的内核栈
#[unsafe(link_section = ".data")]
#[allow(unused)]
#[unsafe(no_mangle)]
pub static mut STACK0: [u8; STACK_SIZE * NCPU] = [0; STACK_SIZE * NCPU];
