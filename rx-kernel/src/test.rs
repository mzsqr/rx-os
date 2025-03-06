#[allow(unused_imports)]
use crate::*;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    use crate::shutdown::{self, RESET_TYPE_SHUTDOWN, SHUTDOWN};

    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    shutdown::system_reset(SHUTDOWN, RESET_TYPE_SHUTDOWN);
}

#[cfg(test)]
pub trait Testable {
    fn run(&self);
}

#[cfg(test)]
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}
