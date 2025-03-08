use core::sync::atomic::AtomicUsize;

use riscv::{
    interrupt::{Exception, Interrupt},
    register::{
        self, scause, sepc, stval,
        stvec::{self, Stvec},
    },
};

use crate::{
    arch::riscv::{
        qemu::layout::{TRAMPOLINE, TRAPFRAME},
        register::sstatus,
    },
    println,
    process::cpu::{CPUManager, cpuid},
};

pub static TICKS: AtomicUsize = AtomicUsize::new(0);
unsafe extern "C" {
    fn kernelvec();
    fn uservec();
    fn trampoline();
    fn userret();
    fn etext();
}
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
            }
            // Clock interrupt
            scause::Trap::Interrupt(Interrupt::SupervisorSoft) => {
                unsafe {
                    if cpuid() == 0 {
                        clock_intr();
                    }
                    register::sip::clear_ssoft();

                    if p.killed() {
                        // exit
                    }

                    p.yielding();
                }
            }
            // user system call
            scause::Trap::Exception(Exception::UserEnvCall) => {
                if p.killed() {
                    // exit
                }

                tf.update_epc(); // skip ecall 

                // An interrupt will change sstatus &c registers,
                // so don't enable until done with those registers.
                unsafe { sstatus::intr_on() };

                // TODO: handle syscall
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
            // TODO: exit
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
pub unsafe extern "C" fn kernel_trap(
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    which: usize,
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
        scause::Trap::Exception(Exception::Breakpoint) => {}
        scause::Trap::Exception(Exception::LoadFault) => {}
        scause::Trap::Exception(Exception::LoadPageFault) => {}
        scause::Trap::Exception(Exception::StorePageFault) => {}
        scause::Trap::Exception(Exception::SupervisorEnvCall) => {}
        scause::Trap::Exception(Exception::InstructionFault) => {}
        scause::Trap::Exception(Exception::InstructionPageFault) => {}
        scause::Trap::Interrupt(Interrupt::SupervisorExternal) => {}
        scause::Trap::Interrupt(Interrupt::SupervisorSoft) => {}
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
    TICKS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
}
