use core::cell::Cell;

use alloc::{boxed::Box, sync::Arc};

use crate::{
    arch::riscv::qemu::layout::PGSIZE,
    lock::{Mutex, MutexGuard},
    memory::{PageAllocator, RawPage},
    println,
    process::{cpu::CPUManager, manager::PROC_MANAGER},
};

use super::file::{FileType, VFile};

const PIPESIZE: usize = 512;

pub struct PipeInner {
    data: [u8; PIPESIZE],
    nread: usize,
    nwrite: usize,
    readopen: bool,
    writeopen: bool,
}

pub type Pipe = Mutex<*mut PipeInner>;

impl Clone for Pipe {
    fn clone(&self) -> Self {
        Mutex::new(unsafe { *self.as_ptr() }, "Pipe")
    }
}

impl Pipe {
    /// 创建一个管道
    /// 分配两个文件结构（读，写）
    pub fn alloc() -> (Arc<VFile>, Arc<VFile>) {
        // leak
        let pi = unsafe { RawPage::new_zeroed() };
        let pi = pi as *mut _ as *mut PipeInner;
        unsafe {
            let pi = &mut *pi;
            pi.readopen = true;
            pi.writeopen = true;
            pi.nread = 0;
            pi.nwrite = 0;
        }
        let pipe = Mutex::new(pi, "Pipe");
        let vread = Arc::new(VFile {
            ftype: FileType::Pipe,
            readable: true,
            writeable: false,
            pipe: Some(pipe.clone()),
            inode: None,
            offset: Cell::new(0),
            major: 0,
        });
        let vwrite = Arc::new(VFile {
            ftype: FileType::Pipe,
            readable: false,
            writeable: true,
            pipe: Some(pipe.clone()),
            inode: None,
            offset: Cell::new(0),
            major: 0,
        });

        (vread, vwrite)
    }

    pub fn close(&self, writeable: bool) {
        let mut g = self.lock();
        let pipe = unsafe { &mut **g };
        if writeable {
            pipe.writeopen = false;
            PROC_MANAGER.wake_up(&pipe.nread as *const _ as usize);
        } else {
            pipe.readopen = false;
            PROC_MANAGER.wake_up(&pipe.nwrite as *const _ as usize);
        }
        if !pipe.readopen && !pipe.writeopen {
            let data = *MutexGuard::leak(g);
            unsafe {
                let _ = Box::from_raw(data as *mut RawPage);
            };
        }
    }

    pub fn write(&self, addr: usize, n: usize) -> Result<usize, &'static str> {
        let proc = unsafe { CPUManager::myproc().unwrap() };
        let pgt = unsafe {
            proc.data
                .as_mut_unchecked()
                .pagetable
                .as_deref_mut()
                .unwrap()
        };

        let mut g = self.lock();
        let mut inner = unsafe { &mut **g };
        let mut i = 0;
        while i < n {
            if !inner.readopen && proc.killed() {
                return Err("This proc is killed");
            }
            if inner.nwrite == inner.nread + PIPESIZE {
                PROC_MANAGER.wake_up(&inner.nread as *const _ as usize);
                proc.sleep(&inner.nwrite as *const _ as usize, g);
                g = self.lock();
                inner = unsafe { &mut **g };
            } else {
                let mut ch = [0u8; 1];
                pgt.copy_in(&mut ch, addr + i)?;
                inner.data[inner.nwrite % PIPESIZE] = ch[0];
                inner.nwrite += 1;
                i += 1;
            }
        }

        PROC_MANAGER.wake_up(&inner.nread as *const _ as usize);
        Ok(i)
    }

    pub fn read(&self, addr: usize, n: usize) -> Result<usize, &'static str> {
        let proc = unsafe { CPUManager::myproc().unwrap() };
        let pgt = unsafe {
            proc.data
                .as_mut_unchecked()
                .pagetable
                .as_deref_mut()
                .unwrap()
        };

        let mut g = self.lock();
        let mut inner = unsafe { &mut **g };
        while inner.nread == inner.nwrite && inner.writeopen {
            if proc.killed() {
                return Err("This proc is killed");
            }
            proc.sleep(&inner.nread as *const _ as usize, g);
            g = self.lock();
            inner = unsafe { &mut **g };
        }
        let mut i = 0;
        while i < n {
            if inner.nread == inner.nwrite {
                break;
            }
            let ch = inner.data[inner.nread % PIPESIZE];
            pgt.copy_out(addr + i, &[ch])?;
            i += 1;
            inner.nread += 1;
        }

        PROC_MANAGER.wake_up(&inner.nwrite as *const _ as usize);
        Ok(i)
    }
}
