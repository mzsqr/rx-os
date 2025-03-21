#![no_std]
#![no_main]

use core::panic::PanicInfo;

use rx_kernel::syscall::exit;

#[unsafe(no_mangle)]
fn main() {
    rx_kernel::syscall::write(1, "hello world\n".as_bytes());
}

#[panic_handler]
fn handle(_info: &PanicInfo) -> ! {
    exit(-1);
    loop {}
}
