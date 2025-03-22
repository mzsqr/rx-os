use crate::trap::uptime;

use super::Syscall;

impl Syscall<'_> {
    pub fn sys_uptime(&self) -> Result<usize, ()> {
        Ok(uptime())
    }
}
