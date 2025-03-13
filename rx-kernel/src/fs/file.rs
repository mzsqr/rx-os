use core::{cell::Cell, ptr::slice_from_raw_parts};

use crate::{
    arch::riscv::qemu::{
        fs::{BSIZE, MAXOPBLOCKS},
        param::NDEV,
    },
    process::cpu::CPUManager,
};

use super::{
    devices::DEVICE_LIST,
    inode::{ICACHE, Inode, InodeCache},
    log::Log,
    stat::Stat,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FileType {
    None = 0,
    Pipe = 1,
    Inode = 2,
    Device = 3,
}

#[derive(Clone)]
pub struct Device();

#[derive(Clone)]
pub struct File();

#[derive(Clone)]
pub enum FileInner {
    Device(Device),
    File(File),
}

#[derive(Debug, Clone)]
pub struct VFile {
    pub ftype: FileType,
    pub readable: bool,
    pub writeable: bool,
    // pipe
    pub inode: Option<Inode>,
    pub offset: Cell<u32>,
    pub major: i16,
}

impl VFile {
    pub const fn init() -> Self {
        Self {
            ftype: FileType::None,
            readable: false,
            writeable: false,
            inode: None,
            offset: Cell::new(0),
            major: 0,
        }
    }

    pub fn readable(&self) -> bool {
        self.readable
    }

    pub fn writeable(&self) -> bool {
        self.writeable
    }

    pub fn read(&self, addr: usize, len: usize) -> Result<usize, &'static str> {
        if !self.readable() {
            panic!("File can't be read");
        }

        match self.ftype {
            FileType::None => panic!("Invalid file!"),
            FileType::Pipe => Err("Not implemented"),
            FileType::Inode => {
                let inode = self.inode.as_ref().unwrap();
                let mut ig = inode.lock();
                let total = ig.read(true, addr, self.offset.get(), len as u32)?;
                self.offset.set(self.offset.get() + total as u32);
                Ok(total)
            }
            FileType::Device => {
                if self.major < 0
                    || self.major as usize > NDEV
                    || unsafe {
                        DEVICE_LIST.table.as_ref_unchecked()[self.major as usize]
                            .read
                            .is_none()
                    }
                {
                    return Err("[Error] vfs: Fail to read file");
                }
                let read =
                    unsafe { DEVICE_LIST.table.as_ref_unchecked()[self.major as usize].read() };
                let ret = read(true, addr, len).ok_or("Failed to read device")?;
                Ok(ret)
            }
        }
    }

    pub fn write(&self, addr: usize, len: usize) -> Result<usize, &'static str> {
        if !self.writeable() {
            panic!("File can't be written");
        }

        match self.ftype {
            FileType::None => panic!("Invalid file"),
            FileType::Pipe => unimplemented!(),
            FileType::Inode => {
                let max = ((MAXOPBLOCKS - 1 - 1 - 2) / 2) * BSIZE;
                let mut count = 0;
                while count < len {
                    let mut write_bytes = len - count;
                    if write_bytes > max {
                        write_bytes = max;
                    }

                    Log::begin_op();
                    let inode = self.inode.as_ref().unwrap();
                    let mut ig = inode.lock();

                    let total =
                        ig.write(true, addr + count, self.offset.get(), write_bytes as u32)?;
                    drop(ig);
                    Log::end_op();

                    self.offset.set(self.offset.get() + total as u32);
                    count += total;
                }
                Ok(count)
            }
            FileType::Device => {
                if self.major < 0
                    || self.major as usize > NDEV
                    || unsafe {
                        DEVICE_LIST.table.as_ref_unchecked()[self.major as usize]
                            .write
                            .is_none()
                    }
                {
                    return Err("[Error] vfs: Fail to write file");
                }
                let write =
                    unsafe { DEVICE_LIST.table.as_ref_unchecked()[self.major as usize].write() };
                let ret = write(true, addr, len).ok_or("Failed to read device")?;
                Ok(ret)
            }
        }
    }

    pub fn stat(&self, addr: usize) -> Result<(), &'static str> {
        let p = unsafe { CPUManager::myproc().unwrap() };
        if self.ftype != FileType::Device && FileType::Inode != self.ftype {
            Err("File type of {:?} not implemented")
        } else {
            let inode = self.inode.as_ref().unwrap();
            let ig = inode.lock();
            let stat = ig.stat();
            drop(ig);

            let pdata = unsafe { p.data.as_mut_unchecked() };
            let pgt = pdata.pagetable.as_deref_mut().unwrap();
            let stat_buf = unsafe {
                &*slice_from_raw_parts(&stat as *const _ as *const u8, size_of::<Stat>())
            };
            pgt.copy_out(addr, stat_buf)?;

            Ok(())
        }
    }
}
