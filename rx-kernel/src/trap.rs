use crate::{
    println,
    shutdown::{self, RESET_REASON_SYSTEM_FAILURE, RESET_TYPE_SHUTDOWN},
};

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
    shutdown::system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
}
