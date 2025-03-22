//! 磁盘上的Inode结构

use core::ptr;

use rx_kernel::fs::InodeType;

use crate::{
    arch::riscv::qemu::fs::{DIRSIZ, IPB, NDIRECT},
    fs::{bio::BCache, log::Log, superblock::SuperBlock},
};

use super::bio::Buf;

#[inline]
fn locate_inode_offset(inum: u32) -> isize {
    inum as isize % IPB as isize
}

/// On-disk inode structure
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiskInode {
    pub itype: InodeType,          // File type
    pub major: i16,                // Major device number (T_REVICE only)
    pub minor: i16,                // Minor device number (T_DEVICE only)
    pub nlink: i16,                // Number of links to inode in file system
    pub size: u32,                 // Size of file (bytes)
    pub addrs: [u32; NDIRECT + 1], // Data block addresses
}

// TODO: align
#[repr(C)]
pub struct DirEntry {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

impl DiskInode {
    pub const fn new() -> Self {
        Self {
            itype: InodeType::Empty,
            major: 0,
            minor: 0,
            nlink: 0,
            size: 0,
            addrs: [0; NDIRECT + 1],
        }
    }

    pub fn try_alloc(&mut self, itype: InodeType) -> Result<(), ()> {
        if self.itype == InodeType::Empty {
            unsafe {
                ptr::write_bytes(self, 0, 1);
            }
            self.itype = itype;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn alloc(dev: u32, itype: InodeType) -> u32 {
        for inum in 1..SuperBlock::size() {
            let (dinode, buf) = Self::find_inode(dev, inum);
            if dinode.try_alloc(itype).is_ok() {
                Log::write(buf);
                return inum;
            }
        }

        panic!("not enough inode to alloc");
    }

    pub fn find_inode(dev: u32, inum: u32) -> (&'static mut DiskInode, Buf<'static>) {
        let blockno = SuperBlock::locate_inode(inum);
        let offset = locate_inode_offset(inum);
        let mut buf = BCache::read(dev, blockno);
        let dinode = unsafe {
            (buf.raw_data_mut() as *mut DiskInode)
                .offset(offset)
                .as_mut()
                .unwrap()
        };

        (dinode, buf)
    }
}

impl DirEntry {
    pub const fn new() -> Self {
        Self {
            inum: 0,
            name: [0; DIRSIZ],
        }
    }
}
