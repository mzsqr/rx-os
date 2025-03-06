//! 虚拟地址和物理地址通过页表进行映射的过程
//!

use crate::arch::riscv::qemu::layout::PGSIZE;

mod pagetable;
mod pagetable_entry;

pub fn page_round_up(addr: usize) -> usize {
    (addr + PGSIZE - 1) & !(PGSIZE - 1)
}

pub fn page_round_down(addr: usize) -> usize {
    addr & !(PGSIZE - 1)
}
