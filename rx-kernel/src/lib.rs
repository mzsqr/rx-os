//! RX-OS的API
//! 基于系统调用的ABI向RUST提供调用接口
//! 系统向外提供的数据结构的定义
//! TODO:
//!     宏生成调用接口
//!
//! 由于在no_std条件下，并且没有重新构建全部标准库
//! 所以需要用户程序自己定义入口函数，接入到start中
//! # Example
//!#[allow(needless_doctest_main)]
//!#[no_run]
//! ```
//! #[unsafe(no_mangle)]
//! fn main() {
//!     rx_kernel::syscall::write(1, "hello world\0".as_bytes());
//! }
//! ```

#![no_std]

use syscall::exit;

pub mod syscall;

unsafe extern "Rust" {
    fn main();
}

#[unsafe(no_mangle)]
fn start() {
    unsafe { main() };
    exit(0);
}
