//! `rx-ox` 是一个模仿xv6的基于Rust实现的内核
//! 目前实现的模块如下：
//!     1. 测试框架
//!     2. 内存管理
//!

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(new_zeroed_alloc)]
#![feature(box_as_ptr)]
#![feature(unsafe_cell_access)]

extern crate alloc;

mod arch;
mod asm;
mod driver;
mod logo;
mod memory;
mod print;
mod process;
mod shutdown;
mod test;
mod trap;

use core::sync::atomic::AtomicBool;

use arch::riscv::qemu::{layout::PGSIZE, param::NCPU};
use logo::LOGO;
use process::cpu;
use riscv::register::{self, medeleg::Medeleg, mideleg::Mideleg, satp::Satp};

static mut TIMER_SCRATCH: [[u64; 5]; NCPU] = [[0u64; 5]; NCPU];
static STARTED: AtomicBool = AtomicBool::new(false);
// 为什么非要把这个连接到数据段才行呢？
// FIXME: Rust的静态变量默认被链接到？
#[unsafe(link_section = ".data")]
#[allow(unused)]
#[unsafe(no_mangle)]
pub static STACK0: [u8; PGSIZE * 4 * 8] = [0; PGSIZE * 4 * 8];

/// # Safety
/// 由entry.S调用
/// 引导启动程序,进行寄存器的初始化操作
#[unsafe(no_mangle)]
pub unsafe fn start() {
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
        register::mideleg::write(Mideleg::from_bits(0xffff));
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

        // register::mie::set_stimer();

        // let mut x = 0_usize;
        // core::arch::asm!("csrr {}, 0x30a", out(reg) x);
        // x |= 1 << 63;
        // core::arch::asm!("csrw 0x30a, {}", in(reg) x);
        // register::mcounteren::set_tm();
        // core::arch::asm!("csrr {}, 0x14d", out(reg) x);
        // x += 1000000;
        // core::arch::asm!("csrw 0x14d, {}", in(reg) x);
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

            memory::kalloc::init();
            #[cfg(test)]
            test_main();
            memory::mapping::kernel_map::init();
            memory::mapping::kernel_map::init_hart();
            process::manager::init();

            STARTED.store(true, core::sync::atomic::Ordering::SeqCst);
        } else {
            while !STARTED.load(core::sync::atomic::Ordering::SeqCst) {
                core::hint::spin_loop()
            }
            println!("hart {} starting\n", cpu::cpuid());
            memory::mapping::kernel_map::init_hart();
        }
    }
    #[allow(clippy::empty_loop)]
    loop {}
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
