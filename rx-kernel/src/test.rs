#[allow(unused_imports)]
use crate::println;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    use crate::shutdown::{self, RESET_TYPE_SHUTDOWN, SHUTDOWN};

    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    shutdown::system_reset(SHUTDOWN, RESET_TYPE_SHUTDOWN);
}
