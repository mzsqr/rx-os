//! 系统中用汇编编写的函数定义

use core::arch::global_asm;

global_asm!(include_str!("entry.S"));
global_asm!(include_str!("kernelvec.S"));
global_asm!(include_str!("switch.S"));
global_asm!(include_str!("trampoline.S"));
