#![no_std]
#![no_main]

mod arch;
mod driver;
mod print;

use core::arch::global_asm;

global_asm!(include_str!("asm/entry.S"));
// global_asm!(include_str!("asm/kernelvec.S"));
global_asm!(include_str!("asm/switch.S"));
global_asm!(include_str!("asm/trampoline.S"));

#[unsafe(no_mangle)]
unsafe fn start() {
    driver::console::init();

    // Now the serial port is ready to be used. To send a byte:
    print!("Hello world");
}
