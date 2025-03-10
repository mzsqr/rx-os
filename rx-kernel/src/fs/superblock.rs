//! 超级块
//! 保存在磁盘最初的内容，描述整个磁盘的结构
//!

use core::{
    cell::{LazyCell, UnsafeCell},
    mem::MaybeUninit,
    ptr::{self, copy_nonoverlapping},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    arch::riscv::qemu::fs::{BPB, FSMAGIC, IPB},
    println,
};

use super::bio::BCache;

/// 磁盘上的超级块的布局
/// Disk layout:
/// [ boot block | super block | log | inode blocks |
///                                          free bit map | data blocks]
///
/// mkfs computes the super block and builds an initial file system. The
/// super block describes the disk layout:
#[repr(C)]
#[derive(Debug)]
struct RawSuperBlock {
    magic: u32,      // FSMAGIC
    size: u32,       // 文件系统镜像大小（块）
    nblocks: u32,    // 数据块的数目
    ninodes: u32,    // inode数目
    nlog: u32,       // log块的数目
    logstart: u32,   //第一个日志块的块号
    inodestart: u32, // 第一个inode块的块号
    bmapstart: u32,  // 第一个空闲位图块的块号
}

/// 内存中的超级快
/// 要实现Lazy static
#[derive(Debug)]
pub struct SuperBlock {
    data: UnsafeCell<RawSuperBlock>,
    initialized: AtomicBool,
}

pub static SUPER_BLOCK: SuperBlock = SuperBlock::uninit();

unsafe impl Sync for SuperBlock {}

impl SuperBlock {
    const fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(RawSuperBlock {
                magic: 0,
                size: 0,
                nblocks: 0,
                ninodes: 0,
                nlog: 0,
                logstart: 0,
                inodestart: 0,
                bmapstart: 0,
            }),
            initialized: AtomicBool::new(false),
        }
    }

    /// 初始化dev号为dev的设备
    /// TODO: 支持多个设备
    pub unsafe fn init(dev: u32) {
        if SUPER_BLOCK.initialized.load(Ordering::Relaxed) {
            return;
        }

        let buf = BCache::read(dev, 1);
        unsafe {
            ptr::copy_nonoverlapping(
                buf.raw_data() as *const RawSuperBlock,
                SUPER_BLOCK.data.as_mut_unchecked(),
                1,
            );
        }

        println!("check magic number");
        if unsafe { SUPER_BLOCK.data.as_ref_unchecked() }.magic != FSMAGIC {
            panic!("invalid file system magic num");
        }
        SUPER_BLOCK.initialized.store(true, Ordering::SeqCst);
    }

    fn read(&self) -> &RawSuperBlock {
        unsafe { self.data.as_ref_unchecked() }
    }

    pub fn read_log() -> (u32, u32) {
        let sb = SUPER_BLOCK.read();

        (sb.logstart, sb.nlog)
    }

    pub fn size() -> u32 {
        let sb = SUPER_BLOCK.read();
        sb.size
    }

    pub fn read_inode() -> (u32, u32) {
        let sb = SUPER_BLOCK.read();

        (sb.inodestart, sb.ninodes)
    }

    pub fn read_bmp() -> u32 {
        SUPER_BLOCK.read().bmapstart
    }

    pub fn locate_inode(inum: u32) -> u32 {
        let sb = SUPER_BLOCK.read();
        if inum >= sb.ninodes {
            panic!(
                "query inum {} larger than maximum inode nums {}",
                inum,
                sb.ninodes - 1
            );
        }

        inum / (IPB as u32) + sb.inodestart
    }

    pub fn locate_bmp(blockno: u32) -> u32 {
        SUPER_BLOCK.read().bmapstart + blockno / BPB
    }
}
