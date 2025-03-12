use core::{
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    arch::riscv::qemu::fs::{BSIZE, LOGSIZE, MAXOPBLOCKS},
    fs::{bio::BufData, superblock::SuperBlock},
    lock::Mutex,
    println,
    process::{
        cpu::CPUManager,
        manager::{PROC_MANAGER, ProcManager},
    },
};

use super::bio::{BCache, Buf};

pub static LOG: Mutex<Log> = Mutex::new(Log::unint(), "Log");

/// 文件系统的Log信息
pub struct Log {
    start: u32, // 文件系统的初始块
    size: u32,  // 可用于日志的剩余块
    dev: u32,
    outstanding: u32, // 同时进行的文件系统调用数目
    commiting: bool,  // 是否有正在提交的文件系统请求
    lh: LogHeader,
}

impl Log {
    pub const fn unint() -> Self {
        Self {
            start: 0,
            size: 0,
            dev: 0,
            outstanding: 0,
            commiting: false,
            lh: LogHeader {
                len: 0,
                blocknos: [0; LOGSIZE - 1],
            },
        }
    }

    pub unsafe fn init(dev: u32) {
        debug_assert!(size_of::<LogHeader>() < BSIZE);
        debug_assert_eq!(align_of::<BufData>() % align_of::<LogHeader>(), 0);

        let (start, nlog) = SuperBlock::read_log();
        let mut g = LOG.lock();
        g.start = start;
        g.size = nlog;
        g.dev = dev;
        g.recover();
    }

    fn recover(&mut self) {
        println!("file system: checking logs");
        self.read_head();
        if self.lh.len > 0 {
            println!("file system: recovering from logs");
            self.install_trans(true);
            self.empty_head();
        } else {
            println!("file system: no need to recover");
        }
    }

    fn read_head(&mut self) {
        let buf = BCache::read(self.dev, self.start);
        unsafe {
            ptr::copy_nonoverlapping(buf.raw_data() as *const LogHeader, &mut self.lh, 1);
        }
    }

    fn write_head(&mut self) {
        let mut buf = BCache::read(self.dev, self.start);
        unsafe {
            ptr::copy_nonoverlapping(&self.lh, buf.raw_data_mut() as *mut LogHeader, 1);
        }
        buf.write();
    }

    fn empty_head(&mut self) {
        self.lh.len = 0;
        let mut buf = BCache::read(self.dev, self.start);
        let raw_lh = buf.raw_data_mut() as *mut LogHeader;
        unsafe {
            raw_lh.as_mut().unwrap().len = 0;
        }
        buf.write();
    }

    fn install_trans(&mut self, recovering: bool) {
        for i in 1..=self.lh.len {
            let log_buf = BCache::read(self.dev, self.start + i);

            let mut disk_buf = BCache::read(self.dev, self.lh.blocknos[i as usize - 1]);

            unsafe {
                ptr::copy(log_buf.raw_data(), disk_buf.raw_data_mut(), 1);
            }
            disk_buf.write();
            if !recovering {
                // pin in write
                unsafe {
                    disk_buf.unpin();
                }
            }
        }
    }

    pub unsafe fn commit() {
        let mut g = LOG.lock();

        if !g.commiting {
            panic!("log : commiting while the commiting flag is not set");
        }
        if g.lh.len > 0 {
            g.write_log();
            g.write_head();
            g.install_trans(false);
            g.empty_head();
        }
    }

    fn write_log(&mut self) {
        for i in 1..=self.lh.len {
            let mut log_buf = BCache::read(self.dev, self.start + i);
            let mut cache_buf = BCache::read(self.dev, self.lh.blocknos[i as usize - 1]);

            unsafe {
                ptr::copy(cache_buf.raw_data(), log_buf.raw_data_mut(), 1);
            }
            log_buf.write();
        }
    }

    pub fn begin_op() {
        let mut g = LOG.lock();
        loop {
            if g.commiting
                || 1 + g.lh.len as usize + (g.outstanding + 1) as usize * MAXOPBLOCKS > LOGSIZE
            {
                let chan = g.deref() as *const Log as usize;
                unsafe {
                    if let Some(p) = CPUManager::myproc() {
                        p.sleep(chan, g);
                    }
                }
                g = LOG.lock();
            } else {
                g.outstanding += 1;
                break;
            }
        }
    }

    pub fn write(buf: Buf<'_>) {
        let mut g = LOG.lock();

        if (g.lh.len + 1) as usize >= LOGSIZE || g.lh.len + 1 >= g.size {
            panic!("log: not enough space for ongoing trasactions");
        }
        if g.outstanding < 1 {
            panic!("log: this log write is out of recording");
        }

        for i in 0..g.lh.len {
            if g.lh.blocknos[i as usize] == buf.read_blockno() {
                return;
            }
        }

        if g.no_space() {
            panic!("log: not enough space for this transaction");
        }

        unsafe {
            // unpin in install trans
            buf.pin();
        }

        let len = g.lh.len as usize;
        g.lh.blocknos[len] = buf.read_blockno();
        g.lh.len += 1;
    }

    fn no_space(&self) -> bool {
        self.lh.len as usize + 2 >= LOGSIZE || self.lh.len + 2 >= self.size
    }

    pub fn end_op() {
        let mut committing = false;

        let mut g = LOG.lock();
        g.outstanding -= 1;
        if g.commiting {
            panic!("logs: end fs op while the log is committing");
        }
        if g.outstanding == 0 {
            g.commiting = true;
            committing = true;
        } else {
            let chan = g.deref() as *const Log as usize;
            PROC_MANAGER.wake_up(chan);
        }
        drop(g);

        if committing {
            unsafe { Log::commit() };
            let mut g = LOG.lock();
            g.commiting = false;
            let channel = g.deref() as *const Log as usize;
            PROC_MANAGER.wake_up(channel);
        }
    }
}

#[repr(C)]
struct LogHeader {
    len: u32,
    blocknos: [u32; LOGSIZE - 1],
}
