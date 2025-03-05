#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod arch;
mod driver;
mod logo;
mod print;
mod process;
mod shutdown;
mod test;

use core::{arch::global_asm, sync::atomic::AtomicBool};

use arch::riscv::qemu::{layout::PGSIZE, param::NCPU};
use logo::LOGO;
use process::cpu;
use riscv::register::{self, medeleg::Medeleg, satp::Satp};

global_asm!(include_str!("asm/entry.S"));
global_asm!(include_str!("asm/kernelvec.S"));
global_asm!(include_str!("asm/switch.S"));
global_asm!(include_str!("asm/trampoline.S"));

static mut TIMER_SCRATCH: [[u64; 5]; NCPU] = [[0u64; 5]; NCPU];
static STARTED: AtomicBool = AtomicBool::new(false);
#[allow(unused)]
#[unsafe(no_mangle)]
pub static STACK0: [u8; PGSIZE * 4 * NCPU] = [0; PGSIZE * 4 * NCPU];

/// # Safety
/// 由entry.S调用
/// 引导启动程序,进行寄存器的初始化操作
#[unsafe(no_mangle)]
pub unsafe fn start() -> ! {
    unsafe {
        // Set M Previlege mode to Supervisor, for mret
        register::mstatus::set_mpp(register::mstatus::MPP::Supervisor);

        // set M Exception Program Counter to main, for mret.
        // requires gcc -mcmodel=medany
        register::mepc::write(rust_main as usize);

        // disable paging for now.
        register::satp::write(Satp::from_bits(0));

        // delegate all interrupts and exceptions to supervisor mode.
        register::medeleg::write(Medeleg::from_bits(0xffff));
        register::medeleg::write(Medeleg::from_bits(0xffff));
        arch::riscv::register::sie::intr_on();

        // configure Physical Memory Protection to give supervisor mode access to all of physical memory.
        register::pmpaddr0::write(0x3fffffffffffff);
        register::pmpcfg0::set_pmp(0, register::Range::TOR, register::Permission::RWX, false);

        // ask for clock interrupts.
        timer_init();

        // keep each CPU's hartid in its tp register, for cpuid().
        let id: usize = register::mhartid::read();
        arch::riscv::register::tp::write(id);

        // switch to supervisor mode and jump to main().
        core::arch::asm!("mret");

        #[allow(clippy::empty_loop)]
        loop {}
    }
}

/// # Safety
/// set up to receive timer interrupts in machine mode,
/// which arrive at timervec in kernelvec.S,
/// which turns them into software interrupts for
/// devintr() in trap.rs.
/// 启动时钟中断
unsafe fn timer_init() {
    unsafe {
        // each CPU has a separate source of timer interrupts.
        let id = register::mhartid::read();

        // ask the CLINT for a timer interrupt.
        let interval = 1000000; // cycles; about 1/10th second in qemu.
        arch::riscv::register::clint::add_mtimecmp(id, interval);

        // prepare information in scratch[] for timervec.
        // scratch[0..2] : space for timervec to save registers.
        // scratch[3] : address of CLINT MTIMECMP register.
        // scratch[4] : desired interval (in cycles) between timer interrupts.
        TIMER_SCRATCH[id][3] = arch::riscv::register::clint::count_mtiecmp(id) as u64;
        TIMER_SCRATCH[id][4] = interval;
        register::mscratch::write(TIMER_SCRATCH[id].as_ptr() as usize);

        // set the machine-mode trap handler.
        unsafe extern "C" {
            fn timervec();
        }

        register::mtvec::write(register::mtvec::Mtvec::from_bits(timervec as usize));

        // enable machine-mode interrupts.
        register::mstatus::set_mie();

        // enable machine-mode timer interrupts.
        register::mie::set_mtimer();
    }
}

/// 这是Rust的入口点
/// 在概念上说，这个函数中涉及到的调用都应该与平台无关
#[unsafe(no_mangle)]
unsafe extern "C" fn rust_main() {
    unsafe {
        if cpu::cpuid() == 0 {
            driver::console::init();

            println!("{}", LOGO);
            println!("rx-os kernel is booting!");

            #[cfg(test)]
            test_main();
        }
    }
}

/// temp
///
#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_trap() {}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
