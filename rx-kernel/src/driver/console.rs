//! 使用UART同主机控制台通信
//!

use crate::lock::Mutex;
use lazy_static::lazy_static;

use crate::arch::riscv::qemu::layout;

lazy_static! {
    // this mmio port address is spcified by qemu
    // so use this unsafe function is safe
    pub static ref UART: Mutex<uart_16550::MmioSerialPort> =
        Mutex::new(unsafe { uart_16550::MmioSerialPort::new(layout::UART0) }, "UART0");
}

pub fn init() {
    UART.lock().init();
}
