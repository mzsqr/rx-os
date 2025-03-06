use core::cell::UnsafeCell;
use riscv::register;

use crate::{
    arch::riscv::{
        qemu::layout::{
            CLINT, E1000_REGS, ECAM, KERNEL_BASE, PGSHIFT, PGSIZE, PHYSTOP, PLIC_BASE, TRAMPOLINE,
            UART0, VIRT_TEST, VIRTIO0,
        },
        register::{satp, sfence_vma},
    },
    memory::{
        RawPage,
        address::{Addr, PhysicalAddress, VirtualAddress},
        kalloc::ALLOCATOR,
        mapping::pagetable_entry::PteFlags,
    },
    println,
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
    pgt: UnsafeCell<PageTable>,
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
    // KERNEL_PAGETABLE.pgt.as_ref_unchecked().debug(3, 0);
}

/// 将页表寄存器改为内核页表地址
/// 启用分页
///
/// # Safety
/// 仅用在内核初始化
pub unsafe fn init_hart() {
    unsafe {
        // sfence_vma();
        let r = KERNEL_PAGETABLE.pgt.as_ref_unchecked().as_satp();
        let s = register::satp::Satp::from_bits(r);
        println!(
            "{:?} {:?} {:#x} {:#x}",
            s.asid(),
            s.mode(),
            s.ppn(),
            KERNEL_PAGETABLE.pgt.as_ref_unchecked().as_addr()
        );
        register::satp::write(s);
        sfence_vma();
        println!("Write satp: {:#x}", r);
    }
}

unsafe fn kernel_map() {
    println!("kernel page map");

    let kp = unsafe { KERNEL_PAGETABLE.pgt.as_mut_unchecked() };

    unsafe {
        kp.kernel_map(
            VirtualAddress::new(VIRT_TEST),
            PhysicalAddress::new(VIRT_TEST),
            PGSIZE * 2,
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
            0x400000,
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
}
