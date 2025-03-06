use crate::arch::riscv::register::tp;

/// # Safety
/// this function should be called after save hartid in tp(per hart local register)
pub unsafe fn cpuid() -> usize {
    unsafe { tp::read() }
}
