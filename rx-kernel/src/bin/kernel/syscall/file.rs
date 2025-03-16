use core::{
    cell::Cell,
    ptr::{null_mut, slice_from_raw_parts, slice_from_raw_parts_mut},
    str::from_utf8,
};

use alloc::{boxed::Box, sync::Arc};
use array_macro::array;
use bit_field::BitField;

use crate::{
    arch::riscv::qemu::{
        fs::{DIRSIZ, OpenMode},
        layout::PGSIZE,
        param::{MAXARG, MAXPATH},
    },
    fs::{
        dinode::InodeType,
        file::{FileType, VFile},
        inode::{self, ICACHE},
        log::Log,
        pipe::Pipe,
    },
    memory::{PageAllocator, RawPage, copy_from_kernel},
    println,
    process::elf::exec,
};

use super::{SysResult, Syscall};

impl Syscall<'_> {
    pub fn sys_dup(&self) -> SysResult {
        let old_fd = self.arg(0);
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let file = pdata.open_files[old_fd].as_ref().unwrap();
        let new_fd = self.process.fd_alloc(Arc::clone(file)).unwrap();
        Ok(new_fd)
    }

    pub fn sys_read(&self) -> SysResult {
        let fd = self.arg(0);
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let file = pdata.open_files[fd].as_ref().unwrap();
        let addr = self.arg(1);
        let len = self.arg(2);
        file.read(addr, len).map_err(|_| ())
    }

    pub fn sys_write(&self) -> SysResult {
        let fd = self.arg(0);
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let file = pdata.open_files[fd].as_ref().unwrap();
        let addr = self.arg(1);
        let len = self.arg(2);
        file.write(addr, len).map_err(|_| ())
    }

    pub fn sys_pipe(&self) -> SysResult {
        let addr = self.arg(0);
        let (rp, wp) = Pipe::alloc();
        let rf = self.process.fd_alloc(rp).map_err(|_| ())? as u32;
        let wf = self.process.fd_alloc(wp).map_err(|_| ())? as u32;
        let src = [rf, wf];
        let src_ref =
            unsafe { &*slice_from_raw_parts(src.as_ptr() as *const u8, size_of_val(&src)) };
        unsafe { copy_from_kernel(addr, src_ref, true).map_err(|_| ())? };
        Ok(0)
    }

    pub fn sys_open(&self) -> SysResult {
        let mut path = [0; MAXPATH];

        let addr = self.arg(0);
        self.copy_from_str(addr, &mut path).unwrap();
        let open_mode = self.arg(1);

        Log::begin_op();
        // TODO: OPEN MODE
        let inode = if open_mode.get_bit(9) {
            match ICACHE.create(&path, InodeType::File, 0, 0) {
                Ok(inode) => inode,
                Err(err) => {
                    Log::end_op();
                    println!("[rx-os] syscall: sys_open: {err:?}");
                    return Err(());
                }
            }
        } else {
            match ICACHE.namei(&path) {
                Some(inode) => {
                    let ig = inode.lock();
                    if ig.dinode.itype == InodeType::Directory
                        && open_mode != OpenMode::RDONLY as usize
                    {
                        drop(ig);
                        Log::end_op();
                        return Err(());
                    }
                    drop(ig);
                    inode
                }
                None => {
                    Log::end_op();
                    return Err(());
                }
            }
        };

        let mut ig = inode.lock();

        let mut file = VFile::init();
        file.ftype = if ig.dinode.itype == InodeType::Device {
            FileType::Device
        } else {
            file.offset = Cell::new(0);
            FileType::Inode
        };

        file.writeable = true;
        file.readable = true;

        if open_mode.get_bit(10) && ig.dinode.itype == InodeType::File {
            ig.truncate(&inode);
        }

        drop(ig);

        Log::end_op();

        file.inode = Some(inode);
        file.writeable = open_mode.get_bit(0) | open_mode.get_bit(1);
        file.readable = !open_mode.get_bit(0) | open_mode.get_bit(1);
        let fd = self.process.fd_alloc(Arc::new(file)).map_err(|_| ())?;
        Ok(fd)
    }

    pub fn sys_exec(&self) -> SysResult {
        // unsafe {
        //     self.process
        //         .data
        //         .as_ref_unchecked()
        //         .pagetable
        //         .as_deref()
        //         .unwrap()
        //         .debug(3, 0)
        // };

        let mut path = [0_u8; MAXPATH];
        let mut argv = [null_mut::<u8>(); MAXARG];
        let path_addr = self.arg(0);
        self.copy_from_str(path_addr, &mut path).map_err(|_| ())?;
        let argv_addr = self.arg(1);

        let mut count = 0;
        loop {
            if count >= argv.len() {
                for i in argv {
                    if !i.is_null() {
                        unsafe {
                            let _ = Box::from_raw(i as *mut RawPage);
                        }
                    }
                }
                // too many  exec arguments
                return Err(());
            }
            let mut buf = [0u8; 8];

            self.copy_from_addr(argv_addr + count * size_of::<usize>(), &mut buf)
                .map_err(|_| ())?;
            let user_arg = usize::from_le_bytes(buf);
            if user_arg == 0 {
                argv[count] = null_mut();
                break;
            }
            let mem = unsafe { RawPage::new_zeroed() };
            argv[count] = mem as *mut _ as *mut u8;
            let buf = unsafe { &mut *slice_from_raw_parts_mut(argv[count], PGSIZE) };
            self.copy_from_str(user_arg, buf).map_err(|_| ())?;
            count += 1;
        }

        let argv_tmp = array![i => argv[i] as *const u8; MAXARG];

        let ret = unsafe { exec(str::from_utf8(&path).unwrap(), &argv_tmp).map_err(|_| ())? };

        for i in argv {
            if !i.is_null() {
                let _ = unsafe { Box::from_raw(i as *mut RawPage) };
            }
        }
        Ok(ret)
    }

    pub fn sys_mknod(&self) -> SysResult {
        let mut path = [0_u8; MAXPATH];
        let major = self.arg(1);
        let minor = self.arg(2);
        Log::begin_op();
        let addr = self.arg(0);
        self.copy_from_str(addr, &mut path).map_err(|_| ())?;
        if let Ok(inode) = ICACHE.create(&path, InodeType::Device, major as i16, minor as i16) {
            Log::end_op();
            drop(inode);
            Ok(0)
        } else {
            Log::end_op();
            Err(())
        }
    }

    pub fn sys_close(&self) -> SysResult {
        let fd = self.arg(0);
        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let _ = pdata.open_files[fd].take();
        Ok(0)
    }

    pub fn sys_fstat(&self) -> SysResult {
        let fd = self.arg(0);
        let stat = self.arg(1);

        let pdata = unsafe { self.process.data.as_mut_unchecked() };
        let file = pdata.open_files[fd].as_ref().unwrap();
        file.stat(stat).map(|_| 0).map_err(|_| ())
    }

    pub fn sys_chdir(&self) -> SysResult {
        let mut path = [0_u8; MAXPATH];
        Log::begin_op();
        let addr = self.arg(0);
        self.copy_from_str(addr, &mut path).map_err(|_| ())?;
        if let Some(inode) = ICACHE.namei(&path) {
            let ig = inode.lock();
            if ig.dinode.itype == InodeType::Directory {
                drop(ig);
                let _ = unsafe { self.process.data.as_mut_unchecked().cwd.replace(inode) };
                Log::end_op();
                return Ok(0);
            }
        }
        Log::end_op();
        Err(())
    }

    // TODO: pipe

    pub fn sys_mkdir(&self) -> SysResult {
        let mut path = [0_u8; MAXPATH];
        Log::begin_op();
        let addr = self.arg(0);
        self.copy_from_str(addr, &mut path).map_err(|_| ())?;
        if ICACHE.create(&path, InodeType::Directory, 0, 0).is_ok() {
            Log::end_op();
            Ok(0)
        } else {
            Log::end_op();
            Err(())
        }
    }

    pub fn sys_unlink(&self) -> SysResult {
        let mut path = [0u8; MAXPATH];
        let mut name = [0u8; DIRSIZ];

        let addr = self.arg(0);
        self.copy_from_str(addr, &mut path).map_err(|_| ())?;

        Log::begin_op();
        let parent = ICACHE.namei_parent(&path, &mut name).ok_or(())?;
        let mut ig = parent.lock();
        if name[0] == b'.' && (name[1] == b'\0' || name[1] == b'.' && name[2] == b'\0') {
            drop(ig);
            Log::end_op();
            return Err(());
        }
        let inode = match ig.dir_lookup(&name) {
            Some(inode) => inode,
            None => {
                drop(ig);
                Log::end_op();
                return Err(());
            }
        };
        let mut ig_cur = inode.lock();
        if ig_cur.dinode.itype == InodeType::Directory && !ig_cur.is_dir_empty() {
            drop(ig);
            drop(ig_cur);
            Log::end_op();
            return Err(());
        }

        if ig_cur.dinode.itype == InodeType::Directory {
            ig.dinode.nlink -= 1;
            ig.update();
        }

        drop(ig);

        ig_cur.dinode.nlink -= 1;
        ig_cur.update();
        drop(ig_cur);

        Log::end_op();
        Ok(0)
    }

    pub fn sys_link(&self) -> SysResult {
        let mut new_path = [0_u8; MAXPATH];
        let mut old_path = [0_u8; MAXPATH];
        let mut name = [0_u8; DIRSIZ];

        let old_addr = self.arg(0);
        let new_addr = self.arg(1);
        self.copy_from_str(old_addr, &mut old_path)
            .map_err(|_| ())?;
        self.copy_from_addr(new_addr, &mut new_path)
            .map_err(|_| ())?;

        Log::begin_op();
        let inode = if let Some(inode) = ICACHE.namei(&old_path) {
            inode
        } else {
            Log::end_op();
            return Err(());
        };

        let mut ig = inode.lock();
        if ig.dinode.itype == InodeType::Directory {
            drop(ig);
            Log::end_op();
            return Err(());
        }

        ig.dinode.nlink += 1;
        let parent = if let Some(inode) = ICACHE.namei_parent(&new_path, &mut name) {
            inode
        } else {
            ig.dinode.nlink -= 1;
            drop(ig);
            Log::end_op();
            return Err(());
        };

        let mut pig = parent.lock();

        if pig.dinode.itype != InodeType::Directory || pig.dir_link(&name, inode.inum).is_ok() {
            drop(pig);
            ig.dinode.nlink -= 1;
            drop(ig);
            Log::end_op();
            return Err(());
        }

        ig.update();
        drop(ig);
        Log::end_op();
        Ok(0)
    }
}
