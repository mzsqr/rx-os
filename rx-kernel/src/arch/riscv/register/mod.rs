use crate::println;

pub mod clint;
pub mod satp;
pub mod sie;
pub mod tp;

#[inline]
// flush the TLB.
pub unsafe fn sfence_vma() {
    println!("flush the TLB");
    unsafe { core::arch::asm!("sfence.vma zero, zero") };
    println!("finish sfence vma");
}
