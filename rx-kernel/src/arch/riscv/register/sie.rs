// Supervisor Interrupt Enable
pub enum SIE {
    SEIE = 1 << 9, // external
    STIE = 1 << 5, // timer
    SSIE = 1 << 1, // software
}

#[inline]
pub unsafe fn read() -> usize {
    riscv::register::sie::read().bits()
}

#[inline]
pub unsafe fn write(x: usize) { unsafe {
    riscv::register::sie::write(riscv::register::sie::Sie::from_bits(x));
}}

/// enable all software interrupts
/// still need to set SIE bit in sstatus
pub unsafe fn intr_on() { unsafe {
    riscv::register::sie::set_sext();
    riscv::register::sie::set_ssoft();
    riscv::register::sie::set_stimer();
}}
