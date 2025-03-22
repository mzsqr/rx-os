#![no_std]
#![no_main]

use core::{panic::PanicInfo, str::from_utf8};

use rx_kernel::syscall::{exit, uptime, write};

#[unsafe(no_mangle)]
fn main() {
    rx_kernel::syscall::write(1, "hello world\n".as_bytes());
    rx_kernel::syscall::mkdir("a\0".as_bytes());
    rx_kernel::syscall::write(1, "system has booted ".as_bytes());
    let mut t = uptime() as usize;
    let mut base = 1;
    while t / base >= 10 {
        base *= 10;
    }
    let mut bs = [0u8; 100];
    let mut i = 0;
    while base > 0 {
        let b = (t / base) as u8;
        t %= base;
        base /= 10;
        bs[i] = b + b'0';
        i += 1;
    }
    write(1, from_utf8(&bs).unwrap().as_bytes());
}

#[panic_handler]
fn handle(_info: &PanicInfo) -> ! {
    exit(-1);
    loop {}
}
