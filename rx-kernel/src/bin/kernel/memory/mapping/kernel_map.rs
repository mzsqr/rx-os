use core::cell::UnsafeCell;

use riscv::asm::sfence_vma_all;

use crate::{
    arch::riscv::{
        qemu::layout::{
            CLINT, E1000_REGS, ECAM, KERNEL_BASE, PGSIZE, PHYSTOP, PLIC_BASE, TRAMPOLINE, UART0,
            VIRT_TEST, VIRTIO0,
        },
        register::{satp, sfence_vma},
    },
    memory::{
        RawPage,
        address::{PhysicalAddress, VirtualAddress},
        mapping::pagetable_entry::PteFlags,
    },
    println,
    process::manager::PROC_MANAGER,
};

use super::pagetable::PageTable;

unsafe extern "C" {
    // define in linker
    fn etext();

    // define in trampoline.S
    fn trampoline();
}

// 内核页表如果不会被同时访问，采用如下抽象

pub struct KernelPageTable {
    pub pgt: UnsafeCell<PageTable>,
}

pub static KERNEL_PAGETABLE: KernelPageTable = KernelPageTable {
    pgt: UnsafeCell::new(PageTable::empty()),
};

unsafe impl Sync for KernelPageTable {}

/// 初始化内核页表
///
/// # Safety
/// 需要在内核启动时初始化
pub unsafe fn init() {
    assert_eq!(size_of::<RawPage>(), PGSIZE);
    assert_eq!(align_of::<RawPage>(), PGSIZE);
    assert_eq!(size_of::<RawPage>(), size_of::<PageTable>());
    assert_eq!(align_of::<RawPage>(), align_of::<PageTable>());

    unsafe { kernel_map() };
    // KERNEL_PAGETABLE.pgt.as_mut_unchecked().look();
    // KERNEL_PAGETABLE.pgt.as_ref_unchecked().debug(3, 0);
}

/// 将页表寄存器改为内核页表地址
/// 启用分页
///
/// # Safety
/// 仅用在内核初始化
pub unsafe fn init_hart() {
    unsafe {
        sfence_vma_all();
        let r = KERNEL_PAGETABLE.pgt.as_mut_unchecked().as_satp();

        satp::write(r);
        sfence_vma();
    }
}

unsafe fn kernel_map() {
    println!("kernel page mapping");

    let kp = unsafe { KERNEL_PAGETABLE.pgt.as_mut_unchecked() };

    unsafe {
        kp.kernel_map(
            VirtualAddress::new(VIRT_TEST),
            PhysicalAddress::new(VIRT_TEST),
            PGSIZE,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(UART0),
            PhysicalAddress::new(UART0),
            PGSIZE,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(VIRTIO0),
            PhysicalAddress::new(VIRTIO0),
            PGSIZE,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(ECAM),
            PhysicalAddress::new(ECAM),
            0x10000000,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(E1000_REGS),
            PhysicalAddress::new(E1000_REGS),
            0x20000,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(CLINT),
            PhysicalAddress::new(CLINT),
            0x10000,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(PLIC_BASE),
            PhysicalAddress::new(PLIC_BASE),
            0x4000000,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(KERNEL_BASE),
            PhysicalAddress::new(KERNEL_BASE),
            etext as usize - KERNEL_BASE,
            PteFlags::R | PteFlags::X,
        );

        kp.kernel_map(
            VirtualAddress::new(etext as usize),
            PhysicalAddress::new(etext as usize),
            PHYSTOP - etext as usize,
            PteFlags::R | PteFlags::W,
        );

        kp.kernel_map(
            VirtualAddress::new(TRAMPOLINE),
            PhysicalAddress::new(trampoline as usize),
            PGSIZE,
            PteFlags::R | PteFlags::X,
        );

        PROC_MANAGER.proc_mapstacks();
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use super::*;
    use crate::arch::riscv::register::satp;
    use crate::{
        arch::riscv::qemu::layout::PHYSTOP,
        memory::{
            address::{Addr, PhysicalAddress, VirtualAddress},
            mapping::{page_round_down, pagetable::PageTable, pagetable_entry::PteFlags},
        },
        rust_main,
    };

    #[test_case]
    fn test_kernel_map() {
        let mut kp = PageTable::unew();
        let kp = Box::leak(kp);
        println!("kernel map here");
        unsafe {
            kp.kernel_map(
                VirtualAddress::new(VIRT_TEST),
                PhysicalAddress::new(VIRT_TEST),
                PGSIZE,
                PteFlags::R | PteFlags::W,
            );

            kp.kernel_map(
                VirtualAddress::new(UART0),
                PhysicalAddress::new(UART0),
                PGSIZE,
                PteFlags::R | PteFlags::W,
            );

            kp.kernel_map(
                VirtualAddress::new(KERNEL_BASE),
                PhysicalAddress::new(KERNEL_BASE),
                etext as usize - KERNEL_BASE,
                PteFlags::R | PteFlags::X,
            );

            kp.kernel_map(
                VirtualAddress::new(etext as usize),
                PhysicalAddress::new(etext as usize),
                PHYSTOP - etext as usize,
                PteFlags::R | PteFlags::W,
            );

            kp.kernel_map(
                VirtualAddress::new(TRAMPOLINE),
                PhysicalAddress::new(trampoline as usize),
                PGSIZE,
                PteFlags::R | PteFlags::X,
            );

            // TODO: 其它进程的内核栈的映射
        }

        // kp.debug(3, 0);

        let addr_of_main = rust_main as usize;
        let main_lookup = kp.translate(VirtualAddress::new(addr_of_main), false);
        assert_eq!(
            page_round_down(addr_of_main),
            main_lookup.unwrap().as_pagetable() as usize
        );

        let addr = kp.as_satp();
        unsafe { satp::write(addr) };
        unsafe { sfence_vma() };
        println!("Write satp");
    }
}
