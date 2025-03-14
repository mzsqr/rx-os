//! Buffer cache.
//!
//! The buffer cache is a linked list of buf structures holding
//! cached copies of disk block contents.  Caching disk blocks
//! in memory reduces the number of disk reads and also provides
//! a synchronization point for disk blocks used by multiple processes.
//!
//! Interface:
//! * To get a buffer for a particular disk block, call bread.
//! * After changing buffer data, call bwrite to write it to disk.
//! * When done with the buffer, call brelse.
//! * Do not use the buffer after calling brelse.
//! * Only one process at a time can use a buffer,
//!   so do not keep them longer than necessary.

use core::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{lock::Mutex, print, println};
use array_macro::array;

use crate::{
    arch::riscv::qemu::fs::{BSIZE, NBUF},
    driver::virtio_disk::DISK,
    lock::{SleepMutex, SleepMutexGuard},
};

pub struct BCache {
    ctrl: Mutex<BufLru>,
    bufs: [BufInner; NBUF],
}

pub static BCACHE: BCache = BCache::new();

impl BCache {
    pub const fn new() -> Self {
        Self {
            ctrl: Mutex::new(BufLru::new(), "Buf LRU"),
            bufs: array![_ => BufInner::new(); NBUF],
        }
    }

    // we initialize data already in compile time

    fn get(dev: u32, blockno: u32) -> Buf<'static> {
        let mut ctrl = BCACHE.ctrl.lock();

        if let Some((idx, rc)) = ctrl.find_cached(dev, blockno) {
            drop(ctrl);
            Buf {
                index: idx,
                dev,
                blockno,
                rc_ptr: rc,
                data: Some(BCACHE.bufs[idx].data.lock()),
            }
        } else if let Some((idx, rc)) = ctrl.recycle(dev, blockno) {
            BCACHE.bufs[idx].valid.store(false, Ordering::Relaxed);
            drop(ctrl);
            Buf {
                index: idx,
                dev,
                blockno,
                rc_ptr: rc,
                data: Some(BCACHE.bufs[idx].data.lock()),
            }
        } else {
            panic!("no usable buffer")
        }
    }

    pub fn read(dev: u32, blockno: u32) -> Buf<'static> {
        let mut b = BCache::get(dev, blockno);
        if !BCACHE.bufs[b.index].valid.load(Ordering::Relaxed) {
            DISK.rw(&mut b, false);
            BCACHE.bufs[b.index].valid.store(true, Ordering::Relaxed);
        }

        b
    }

    pub fn release(&self, index: usize) {
        self.ctrl.lock().move_if_no_ref(index);
    }
}

pub struct Buf<'a> {
    index: usize,
    dev: u32,
    blockno: u32,
    rc_ptr: *mut usize,
    data: Option<SleepMutexGuard<'a, BufData>>,
}

impl<'a> Buf<'a> {
    pub unsafe fn uninit(data: SleepMutexGuard<'a, BufData>) -> Self {
        Self {
            index: 0,
            dev: 0,
            blockno: 0,
            rc_ptr: null_mut(),
            data: Some(data),
        }
    }

    pub fn read_blockno(&self) -> u32 {
        self.blockno
    }

    pub fn write(&mut self) {
        DISK.rw(self, true);
    }

    pub fn raw_data(&self) -> *const BufData {
        self.data.as_deref().unwrap()
    }

    pub fn raw_data_mut(&mut self) -> *mut BufData {
        self.data.as_deref_mut().unwrap()
    }

    /// Pin the buf.
    /// SAFETY: it should be definitly safe.
    ///     Because the current refcnt >= 1, so the rc_ptr is valid.
    pub unsafe fn pin(&self) {
        unsafe {
            let rc = *self.rc_ptr;
            *self.rc_ptr = rc + 1;
        }
    }

    /// Unpin the buf.
    /// SAFETY: it should be called matching pin.
    pub unsafe fn unpin(&self) {
        unsafe {
            let rc = *self.rc_ptr;
            if rc <= 1 {
                panic!("buf unpin not match");
            }
            *self.rc_ptr = rc - 1;
        }
    }
}

impl Drop for Buf<'_> {
    fn drop(&mut self) {
        self.data.take();
        BCACHE.release(self.index);
    }
}

struct BufLru {
    inner: [BufCtrl; NBUF],
    head: usize,
    tail: usize,
}

impl BufLru {
    pub const fn new() -> Self {
        let mut s = Self {
            inner: array![idx => BufCtrl::new(idx); NBUF],
            head: 0,
            tail: NBUF - 1,
        };

        s.inner[0].prev = NBUF - 1;
        s.inner[NBUF - 1].next = 0;

        s
    }

    /// 寻找一个指定的块
    /// 存在则返回数据所在索引以及引用计数
    fn find_cached(&mut self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        let mut b = self.head;
        loop {
            let bref = &mut self.inner[b];
            if bref.dev == dev && bref.blockno == blockno {
                bref.refcnt += 1;
                return Some((bref.index, &mut bref.refcnt));
            }
            b = bref.next;
            if b == self.head {
                return None;
            }
        }
    }

    fn recycle(&mut self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        let mut b = self.tail;
        loop {
            let bref = &mut self.inner[b];
            if bref.refcnt == 0 {
                bref.refcnt += 1;
                bref.dev = dev;
                bref.blockno = blockno;
                return Some((bref.index, &mut bref.refcnt));
            }
            b = bref.prev;
            if b == self.tail {
                return None;
            }
        }
    }

    fn move_if_no_ref(&mut self, index: usize) {
        self.inner[index].refcnt -= 1;
        let bref = &self.inner[index];

        if bref.refcnt == 0 && index != self.head {
            if self.tail == index {
                self.tail = bref.prev;
            }

            let prev = bref.prev;
            let next = bref.next;
            self.inner[prev].next = next;
            self.inner[next].prev = prev;

            self.inner[index].prev = self.tail;
            self.inner[index].next = self.head;
            self.inner[self.head].prev = index;
            self.inner[self.tail].next = index;
            self.head = index;
        }
    }
}

struct BufCtrl {
    dev: u32,
    blockno: u32,
    prev: usize,
    next: usize,
    refcnt: usize,
    index: usize,
}

impl BufCtrl {
    const fn new(idx: usize) -> Self {
        Self {
            dev: 0,
            blockno: 0,
            prev: if idx == 0 { 0 } else { idx - 1 },
            next: idx + 1,
            refcnt: 0,
            index: idx,
        }
    }
}

struct BufInner {
    // valid is guarded by
    // the bcache spinlock and the relevant buf sleeplock
    // holding either of which can get access to them
    valid: AtomicBool,
    data: SleepMutex<BufData>,
}

impl BufInner {
    const fn new() -> Self {
        Self {
            valid: AtomicBool::new(false),
            data: SleepMutex::new(BufData::new(), "BufData"),
        }
    }
}

/// Alignment of BufData should suffice for other structs
/// that might converts from this struct.
#[repr(C, align(8))]
pub struct BufData([u8; BSIZE]);

impl BufData {
    pub const fn new() -> Self {
        Self([0; BSIZE])
    }
}
