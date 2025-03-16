use core::{fmt, panic::PanicInfo};

use crate::{driver::uart::UART, shutdown::shutdown};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    UART.lock().write_fmt(args).unwrap();
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
    shutdown();
    loop {}
}

#[unsafe(no_mangle)]
fn abort() -> ! {
    panic!("abort");
}

///
/// implement a macro like std::dbg
#[macro_export]
#[allow(unused_macros)]
macro_rules! dbg {
    () => {
        println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { $crate::dbg!($val) };
    ((val:expr),+ $(,)?) => {
        (($crate::dbg!($val)),+,)
    };
}

/// like `std::dbg` macro（16 Hexadecimal）
#[macro_export]
#[allow(unused_macros)]
macro_rules! dbgx {
    () => {
        println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                println!("[{}:{}] {} = {:#x?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { dbgx!($val) };
    ((val:expr),+ $(,)?) => {
        ($(dbgx!($val)),+,)
    };
}
