use core::ops::Add;

use riscv::{
    interrupt::{Exception, Interrupt},
    register::{
        self, scause, sepc, stval,
        stvec::{self, Stvec},
    },
};

use crate::{
    arch::riscv::{
        qemu::layout::{TRAMPOLINE, TRAPFRAME, UART0_IRQ, VIRTIO0_IRQ},
        register::sstatus,
    },
    asm::{kernelvec, trampoline, userret, uservec},
    driver::{
        plic::{plic_claim, plic_complete},
        uart::UART,
        virtio_disk::DISK,
    },
    lock::Mutex,
    println,
    process::{
        cpu::{self, CPUManager, cpuid},
        exit,
    },
    shutdown::{
        REBOOT, RESET_REASON_NO_REASON, RESET_TYPE_COLD_REBOOT, RESET_TYPE_SHUTDOWN, SHUTDOWN,
        system_reset,
    },
    syscall::syscall_handler,
};

pub static TICKS: Mutex<usize> = Mutex::new(0, "TICKS");
/// 配置内核的异常处理程序
pub unsafe fn init_hart() {
    unsafe { stvec::write(Stvec::from_bits(kernelvec as usize)) };
}

#[unsafe(no_mangle)]
pub unsafe fn user_trap() {
    let sepc = sepc::read();
    let scause = scause::read();

    if !unsafe { sstatus::is_from_user() } {
        panic!("user_trap(): not from user mode.")
    }

    unsafe { stvec::write(Stvec::from_bits(kernelvec as usize)) };

    if let Some(p) = unsafe { CPUManager::myproc() } {
        let tf = unsafe { &mut (*p.data.as_mut_unchecked().get_trapframe()) };
        tf.epc = sepc;

        match scause.cause().try_into().unwrap() {
            // Device interrupt
            scause::Trap::Interrupt(Interrupt::SupervisorExternal) => {
                // use plic claim
                // from UART0
                // from VIRTIO
                if let Some(interrupt) = plic_claim() {
                    match interrupt {
                        VIRTIO0_IRQ => {
                            DISK.lock().intr();
                        }

                        UART0_IRQ => {
                            UART.intr();
                        }
                        _ => {
                            panic!("Unresolved interrupt");
                        }
                    }
                    plic_complete(interrupt);
                }
            }
            // Clock interrupt
            scause::Trap::Interrupt(Interrupt::SupervisorSoft) => unsafe {
                if cpuid() == 0 {
                    clock_intr();
                }
                register::sip::clear_ssoft();

                if p.killed() {
                    exit(-1);
                }

                p.yielding();
            },
            // user system call
            scause::Trap::Exception(Exception::UserEnvCall) => {
                if p.killed() {
                    exit(-1);
                }

                tf.update_epc(); // skip ecall 

                // An interrupt will change sstatus &c registers,
                // so don't enable until done with those registers.
                unsafe { sstatus::intr_on() };

                syscall_handler();
            }
            _ => {
                println!(
                    "usertrap: unexpected scacuse: {:?}\n pid: {}",
                    scause.cause(),
                    p.pid()
                );
                println!("sepc: 0x{:x}, stval: 0x{:x}", sepc, stval::read());
                p.set_killed(true);
            }
        }
        if p.killed() {
            exit(-1);
        }

        unsafe { user_trap_ret() }
        // exit
    }
}

#[unsafe(no_mangle)]
pub unsafe fn user_trap_ret() {
    if let Some(p) = unsafe { CPUManager::myproc() } {
        unsafe {
            sstatus::intr_off();
            stvec::write(Stvec::from_bits(
                TRAMPOLINE + (uservec as usize - trampoline as usize),
            ));
        };

        // 配置trapframe的值
        // 下次uservec可以再次进入内核
        let pdata = unsafe { p.data.as_mut_unchecked() };
        pdata.user_init();

        let mut st = unsafe { sstatus::read() };
        st = sstatus::clear_spp(st);
        st = sstatus::user_intr_on(st);
        unsafe { sstatus::write(st) };

        unsafe {
            sepc::write((*pdata.trapframe).epc);
        }

        let satp = pdata.pagetable.as_ref().map(|p| p.as_satp()).unwrap();

        let userret_virt = TRAMPOLINE + (userret as usize - trampoline as usize);
        unsafe {
            let userret_virt: unsafe extern "C" fn(usize, usize) -> ! =
                core::mem::transmute(userret_virt);
            userret_virt(TRAPFRAME, satp);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn kernel_trap(
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    which: usize, // save in x17
) {
    let sepc = sepc::read();
    let st = unsafe { sstatus::read() };
    let scause = scause::read();
    let stval = stval::read();

    if !unsafe { sstatus::is_from_supervisor() } {
        panic!("Not from supervisor mode");
    }

    if unsafe { sstatus::intr_get() } {
        panic!("kernel_trap(): interrupts enabled");
    }

    let mut local_spec = sepc;
    match scause.cause().try_into().unwrap() {
        scause::Trap::Exception(Exception::Breakpoint) => {
            local_spec += 2;
            println!("Breakpoint");
        }
        scause::Trap::Exception(Exception::LoadFault) => panic!("Load Fault"),
        scause::Trap::Exception(Exception::LoadPageFault) => {
            panic!(
                "[Panic] Load Page Fault!\n stval: {:#x}\n sepc: {:#x}\n",
                stval, sepc
            );
        }
        scause::Trap::Exception(Exception::StorePageFault) => {
            panic!(
                "[Panic] Store Page Fault!\n stval: {:#x}\n sepc: {:#x}\n",
                stval, sepc
            );
        }
        scause::Trap::Exception(Exception::SupervisorEnvCall) => match which {
            SHUTDOWN => {
                println!("\x1b[1;31mShutdown!\x1b[0m");
                system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
            }

            REBOOT => {
                println!("\x1b[1;31mReboot!\x1b[0m");
                system_reset(RESET_TYPE_COLD_REBOOT, RESET_REASON_NO_REASON);
            }

            _ => {
                panic!("Unresolved Kernel syscall");
            }
        },
        scause::Trap::Exception(Exception::InstructionFault) => {
            panic!("Instruction Fault, sepc: 0x{:x}", sepc)
        }
        scause::Trap::Exception(Exception::InstructionPageFault) => {
            panic!(
                "[Panic] Instruction Page Fault: sepc: {:#x} stval: {:#x}",
                sepc, stval
            );
        }
        scause::Trap::Interrupt(Interrupt::SupervisorExternal) => {
            // 设备中断
            // like user_trap
            if let Some(interrupt) = plic_claim() {
                match interrupt {
                    VIRTIO0_IRQ => {
                        DISK.lock().intr();
                    }

                    UART0_IRQ => {
                        UART.intr();
                    }
                    _ => {
                        panic!("Unresolved interrupt");
                    }
                }
                plic_complete(interrupt);
            }
        }
        scause::Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // 时钟中断
            unsafe {
                if cpu::cpuid() == 0 {
                    clock_intr();
                }
            }

            unsafe { register::sip::clear_ssoft() };

            unsafe { CPUManager::mycpu() }.try_yield_proc();
        }
        _ => {
            panic!("Unresolved Trap!")
        }
    }

    unsafe {
        sepc::write(local_spec);
        sstatus::write(st);
    }
}

pub unsafe fn clock_intr() {
    if unsafe { cpuid() == 0 } {
        *TICKS.lock() += 1;
    }
}

/// in seconds
pub fn uptime() -> usize {
    *TICKS.lock() / 10
}
