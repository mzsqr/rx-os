use core::cell::UnsafeCell;

use crate::memory::address::{PhysicalAddress, VirtualAddress};

use super::page_round_down;

#[repr(C, align(0x1000))]
pub struct PageTable([usize; 512]);

pub struct SafePageTable(UnsafeCell<PageTable>);

static PAGETABLE: SafePageTable = SafePageTable(UnsafeCell::new(PageTable::empty()));

unsafe impl Sync for SafePageTable {}

impl PageTable {
    pub const fn empty() -> Self {
        Self([0; 512])
    }

    pub fn kernel_map(&mut self, va: usize, pa: usize, size: usize, perm: usize) {
        // find entry
        let mut va = page_round_down(va);
        let mut pa = page_round_down(pa);
        let end = va + size;

        while va < end {}
    }
}
