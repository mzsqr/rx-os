use core::{fmt, panic::PanicInfo};

use crate::driver;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    driver::console::UART.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($( $arg:tt )*) => {
        $crate::print::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("\x1b[1;31mpanic: '{}'\x1b[0m", info);
    loop {}
}

#[unsafe(no_mangle)]
fn abort() -> ! {
    panic!("abort");
}
