//! 在内存中缓存的Inode表
//!

use core::{
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
    str::from_utf8,
};

use array_macro::array;

use crate::{
    arch::riscv::qemu::fs::{BSIZE, DIRSIZ, NDIRECT, NINDIRECT, NINODE, ROOTDEV, ROOTINUM},
    lock::{Mutex, SleepMutex, SleepMutexGuard},
    memory::{copy_from_kernel, copy_to_kernel},
    println,
    process::cpu::{CPUManager, cpuid},
};

use super::{
    bio::BCache,
    bitmap::BitMap,
    dinode::{DirEntry, DiskInode, InodeType},
    log::Log,
    stat::Stat,
    superblock::SuperBlock,
};

pub struct InodeCache {
    meta: Mutex<[InodeMeta; NINODE]>,
    data: [SleepMutex<InodeData>; NINODE],
}

pub static ICACHE: InodeCache = InodeCache::new();

impl InodeCache {
    const fn new() -> Self {
        Self {
            meta: Mutex::new(array![_ => InodeMeta::new(); NINODE], "Inode Meta"),
            data: array![_=> SleepMutex::new(InodeData::new(), "Inode data"); NINODE],
        }
    }

    /// 增加Inode的引用计数
    fn dup(inode: &Inode) -> Inode {
        inode.clone()
    }

    /// 当前在内存中的Inode使用完毕
    /// 如果这是最后一个引用则会被回收
    /// 由Inode的Drop调用
    fn put(&self, inode: &mut Inode) {
        let mut g = self.meta.lock();
        let i = inode.index;
        let imeta = &mut g[i];

        if imeta.refs == 1 {
            let mut idata = self.data[i].lock();
            if !idata.valid || idata.dinode.nlink > 0 {
                idata.valid = false;
                drop(idata);
                imeta.refs -= 1;
                drop(g);
            } else {
                drop(g);
                idata.dinode.itype = InodeType::Empty;
                idata.truncate(inode);
                idata.valid = false;
                drop(idata);

                let mut g = self.meta.lock();
                g[i].refs -= 1;
                drop(g);
            }
        } else {
            imeta.refs -= 1;
            drop(g);
        }
    }

    /// 在给定设备上分配一个Inode
    /// 根据其给定类型在BitMap标记其为已分配
    /// 返回一个解锁但以分配的Inode引用（————Inode类型是引用）
    pub fn alloc(&self, dev: u32, itype: InodeType) -> Option<Inode> {
        let (_, ninodes) = SuperBlock::read_inode();

        for inum in 1..=ninodes {
            let (dinode, block) = DiskInode::find_inode(dev, inum);
            if dinode.try_alloc(itype).is_ok() {
                Log::write(block);
                return Some(self.get(dev, inum));
            }
        }

        None
    }

    /// 从InodeCache中查找一个Inode
    /// 如果找到则返回对应引用
    /// 否在分配一个内存位置给它
    /// 不会从磁盘中读取
    fn get(&self, dev: u32, inum: u32) -> Inode {
        let mut g = self.meta.lock();

        if let Some((idx, _)) = g
            .iter_mut()
            .enumerate()
            .find(|(_, x)| x.inum == inum && x.refs > 0 && x.dev == dev)
        {
            g[idx].refs += 1;
            Inode {
                dev,
                inum,
                index: idx,
            }
        } else if let Some((index, _)) = g.iter_mut().enumerate().find(|(_, x)| x.refs == 0) {
            // 未在cache中找到inum指向inode信息
            g[index].dev = dev;
            g[index].inum = inum;
            g[index].refs = 1;
            let idata = self.data[index].lock();
            assert!(!idata.valid, "Empty cache is valid here");
            Inode { dev, inum, index }
        } else {
            panic!("inode get: not enough cache.");
        }
    }

    fn namex(&self, path: &[u8], name: &mut [u8; DIRSIZ], is_parent: bool) -> Option<Inode> {
        let mut inode = if path[0] == b'/' {
            self.get(ROOTDEV, ROOTINUM)
        } else {
            let p = unsafe { CPUManager::myproc() }.unwrap();
            InodeCache::dup(unsafe { p.data.as_mut_unchecked() }.cwd.as_ref().unwrap())
        };
        let mut cur = 0;
        loop {
            cur = skip_path(path, cur, name);
            if cur == 0 {
                break;
            }

            let mut data_g = inode.lock();
            if data_g.dinode.itype != InodeType::Directory {
                return None;
            }

            if is_parent && path[cur] == 0 {
                drop(data_g);
                return Some(inode);
            }

            if let Some(last_inode) = data_g.dir_lookup(name) {
                drop(data_g);
                inode = last_inode;
            } else {
                return None;
            }
        }

        if is_parent {
            println!("[Kernel] Warning: namex querying root inode's parent");
            None
        } else {
            Some(inode)
        }
    }

    pub fn namei(&self, path: &[u8]) -> Option<Inode> {
        let mut name: [u8; DIRSIZ] = [0; DIRSIZ];
        self.namex(path, &mut name, false)
    }

    pub fn namei_parent(&self, path: &[u8], name: &mut [u8; DIRSIZ]) -> Option<Inode> {
        self.namex(path, name, true)
    }

    pub fn create(
        &self,
        path: &[u8],
        itype: InodeType,
        major: i16,
        minor: i16,
    ) -> Result<Inode, &str> {
        let mut name = [0_u8; DIRSIZ];
        let dirinode = self.namei_parent(path, &mut name).unwrap();
        let mut dirinode_g = dirinode.lock();

        if let Some(inode) = dirinode_g.dir_lookup(&name) {
            drop(dirinode_g);
            let inode_g = inode.lock();
            if (inode_g.dinode.itype == InodeType::Device
                || inode_g.dinode.itype == InodeType::File)
                && itype == InodeType::File
            {
                drop(inode_g);
                return Ok(inode);
            }
            return Err("create: unmatched type");
        }

        let dev = dirinode_g.dev;
        let inum = DiskInode::alloc(dev, itype);
        let inode = self.get(dev, inum);

        let mut inode_g = inode.lock();
        inode_g.dinode.itype = itype;
        inode_g.dinode.major = major;
        inode_g.dinode.minor = minor;
        inode_g.dinode.nlink = 1;
        inode_g.update();

        if itype == InodeType::Directory {
            inode_g.dinode.nlink += 1;
            inode_g.update();
            inode_g.dir_link(".".as_bytes(), inode.inum)?;
            inode_g.dir_link("..".as_bytes(), inode.inum)?;
        }

        dirinode_g
            .dir_link(&name, inode_g.inum)
            .expect("parent inode fail to link");

        drop(inode_g);
        Ok(inode)
    }
}

/// Skip the path starting at cur by b'/'s.
/// It will copy the skipped content to name.
/// Return the current offset after skiping.
fn skip_path(path: &[u8], mut cur: usize, name: &mut [u8; DIRSIZ]) -> usize {
    // skip preceding b'/'
    while path[cur] == b'/' {
        cur += 1;
    }
    if path[cur] == 0 {
        return 0;
    }

    let start = cur;
    while path[cur] != b'/' && path[cur] != 0 {
        cur += 1;
    }

    let mut count = cur - start;
    if count >= name.len() {
        debug_assert!(false);
        count = name.len() - 1;
    }
    name[..count].copy_from_slice(&path[start..start + count]);
    name[count] = 0;

    // skip succeeding b'/'
    while path[cur] == b'/' {
        cur += 1;
    }
    cur
}

struct InodeMeta {
    dev: u32,
    blockno: u32,
    inum: u32,
    refs: usize,
}

impl InodeMeta {
    const fn new() -> Self {
        Self {
            dev: 0,
            blockno: 0,
            inum: 0,
            refs: 0,
        }
    }
}

pub struct InodeData {
    pub valid: bool,
    pub dev: u32,
    pub inum: u32,
    pub dinode: DiskInode,
}

impl InodeData {
    const fn new() -> Self {
        Self {
            valid: false,
            dev: 0,
            inum: 0,
            dinode: DiskInode::new(),
        }
    }

    /// 返回Inode状态
    pub fn stat(&self) -> Stat {
        Stat {
            dev: self.dev,
            inum: self.inum,
            itype: self.dinode.itype,
            nlink: self.dinode.nlink,
            size: self.dinode.size as usize,
        }
    }

    /// 将Inode指向的数据删除
    ///     元信息中的长度置零
    ///     一级、二级地址指向的块恢复可用
    ///     二级地址块恢复可用
    pub fn truncate(&mut self, inode: &Inode) {
        for addr in &mut self.dinode.addrs[..NDIRECT] {
            if *addr > 0 {
                BitMap::free(inode.dev, *addr);
                *addr = 0;
            }
        }

        if self.dinode.addrs[NDIRECT] > 0 {
            let buf = BCache::read(inode.dev, self.dinode.addrs[NDIRECT]);
            let addrs = unsafe { &*slice_from_raw_parts(buf.raw_data() as *mut u32, NINDIRECT) };
            for addr in addrs {
                if *addr > 0 {
                    BitMap::free(inode.dev, *addr);
                }
            }
            drop(buf);
            BitMap::free(inode.dev, self.dinode.addrs[NDIRECT]);
            self.dinode.addrs[NDIRECT] = 0;
        }

        self.dinode.size = 0;
        self.update();
    }

    /// 将内存中的Inode信息同步到磁盘中
    /// 通常在改变Inode元信息后使用
    pub fn update(&mut self) {
        let (dinode, buf) = DiskInode::find_inode(self.dev, self.inum);
        *dinode = self.dinode;
        Log::write(buf);
    }

    /// 返回Inode指向的数据的第n个数据块的块号
    ///
    /// 如果没有对应的块会自动分配
    /// TODO: Use user defined Result
    pub fn map(&mut self, offset_bn: u32) -> Result<u32, &'static str> {
        let mut addr = 0;
        let mut offset_bn = offset_bn as usize;
        if offset_bn < NDIRECT {
            if self.dinode.addrs[offset_bn] == 0 {
                addr = BitMap::alloc(self.dev);
                self.dinode.addrs[offset_bn] = addr;
            } else {
                addr = self.dinode.addrs[offset_bn];
            }
        } else if offset_bn < NINDIRECT + NDIRECT {
            offset_bn -= NDIRECT;
            if self.dinode.addrs[NDIRECT] != 0 {
                let addr_buf_num = BitMap::alloc(self.dev);
                self.dinode.addrs[NDIRECT] = addr_buf_num;
            }

            let addr_block = BCache::read(self.dev, self.dinode.addrs[NDIRECT]);
            let addr_ref = unsafe { &mut *(addr_block.raw_data() as *mut u32).add(offset_bn) };
            if *addr_ref == 0 {
                *addr_ref = BitMap::alloc(self.dev);
                addr = *addr_ref;
                Log::write(addr_block);
            }
        }

        Ok(addr)
    }

    /// 从Inode中读取数据，并且存入由dst指向的地址中
    pub fn read(
        &mut self,
        is_user: bool,
        mut dst: usize,
        offset: u32,
        count: u32,
    ) -> Result<usize, &'static str> {
        let end = offset.checked_add(count).ok_or("Failed to add count.")?;
        if end > self.dinode.size {
            return Err("inode read: end is more than diskinode's size");
        }

        let mut total = 0;
        let count = count as usize;
        let mut offset = offset as usize;

        // 将offset转为块号和块内偏移量
        // -- map
        //  直接地址
        //  间接地址
        while total < count {
            // 读取当前块
            let block_base = offset / BSIZE;
            let block_offset = offset % BSIZE;
            let len = count - total;
            let block_no = self.map(block_base as u32)?;
            let buf = BCache::read(self.dev, block_no);
            let write_len = len.min(BSIZE);
            let src = unsafe {
                &(*slice_from_raw_parts(buf.raw_data() as *const u8, BSIZE))[block_offset..]
            };
            // println!("{:?}", &src[..write_len]);
            // 复制到dst指向的虚拟地址
            copy_from_kernel(dst, src, is_user, write_len)?;
            offset += write_len;
            total += write_len;
            dst += write_len;
        }

        Ok(total)
    }

    /// 将数据写入Inode中，从src中读取count个字节
    pub fn write(
        &mut self,
        is_user: bool,
        mut src: usize,
        offset: u32,
        count: u32,
    ) -> Result<usize, &'static str> {
        let mut total = 0;
        let count = count as usize;
        let mut offset = offset as usize;

        // 将offset转为块号和块内偏移量
        // -- map
        //  直接地址
        //  间接地址
        while total < count {
            // 读取当前块
            let block_base = offset / BSIZE;
            let block_offset = offset % BSIZE;
            let len = count - total;
            let block_no = self.map(block_base as u32)?;
            let mut buf = BCache::read(self.dev, block_no);
            let read_len = len.min(BSIZE);
            let dst = unsafe {
                &mut (*slice_from_raw_parts_mut(buf.raw_data_mut() as *mut u8, BSIZE))
                    [block_offset..]
            };
            // 从src复制到缓存块中
            copy_to_kernel(dst, src, is_user, read_len)?;
            offset += read_len;
            total += read_len;
            src += read_len;

            Log::write(buf);
        }

        if self.dinode.size < offset as u32 {
            self.dinode.size = offset as u32;
        }
        self.update();

        Ok(total)
    }

    /// 根据给定的名字从当前Inode指向的目录中搜索名字
    ///
    /// # Panics
    /// 如果当前Inode不是目录则会引发panic
    pub fn dir_lookup(&mut self, name: &[u8]) -> Option<Inode> {
        if self.dinode.itype != InodeType::Directory {
            panic!("dirlookup: inode is not a directory");
        }

        let de_size = size_of::<DirEntry>();
        let mut dir_entry = DirEntry::new();
        for offset in (0..self.dinode.size).step_by(de_size) {
            self.read(
                false,
                &mut dir_entry as *mut DirEntry as usize,
                offset,
                de_size as u32,
            )
            .expect("Cannot read entry in this dir");

            if dir_entry.inum == 0 {
                continue;
            }

            // println!(
            //     "{} {}",
            //     from_utf8(&dir_entry.name).unwrap(),
            //     from_utf8(name).unwrap()
            // );

            for (&a, b) in name.iter().zip(dir_entry.name) {
                if a != b {
                    break;
                }
                if b == 0 && a == 0 {
                    return Some(ICACHE.get(self.dev, dir_entry.inum as u32));
                }
            }
        }
        None
    }

    /// 向当前Inode指向的目录中写入一个目录项
    ///
    /// # Panics
    /// 如果当前Inode不是目录
    pub fn dir_link(&mut self, name: &[u8], inum: u32) -> Result<(), &'static str> {
        if self.dinode.itype != InodeType::Directory {
            panic!("dirlookup: inode is not a directory");
        }

        let de_size = size_of::<DirEntry>();
        let mut dir_entry = DirEntry::new();
        let mut entry_offset = 0;
        for offset in (0..self.dinode.size).step_by(de_size) {
            self.read(
                false,
                &mut dir_entry as *mut DirEntry as usize,
                offset,
                de_size as u32,
            )
            .expect("Cannot read entry in this dir");

            entry_offset += de_size;
            if dir_entry.inum == 0 {
                break;
            }
        }
        let mut name_own = [0_u8; DIRSIZ];
        name_own[..name.len()].copy_from_slice(name);
        let dir = DirEntry {
            inum: inum as u16,
            name: name_own,
        };
        self.write(
            false,
            &dir as *const _ as usize,
            entry_offset as u32,
            de_size as u32,
        )?;

        Ok(())
    }

    /// 查询当前Inode指向的目录是否为空
    pub fn is_dir_empty(&mut self) -> bool {
        let mut dir_entry = DirEntry::new();
        let init_size = (2 * size_of::<DirEntry>()) as u32;
        let de_size = size_of::<DirEntry>();
        for offset in (init_size..self.dinode.size).step_by(de_size) {
            if self
                .read(
                    false,
                    &mut dir_entry as *mut _ as usize,
                    offset as u32,
                    de_size as u32,
                )
                .is_err()
            {
                panic!("is_dir_empty: failed to read dir content");
            }

            if dir_entry.inum != 0 {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct Inode {
    pub dev: u32,
    pub inum: u32,
    pub index: usize, // 指向Cache中Inode所在的位置
}

impl Inode {
    pub fn lock(&self) -> SleepMutexGuard<InodeData> {
        let mut g = ICACHE.data[self.index].lock();

        if !g.valid {
            let (dinode, _) = DiskInode::find_inode(self.dev, self.inum);
            g.dinode = *dinode;
            g.valid = true;
            g.dev = self.dev;
            g.inum = self.inum;
            if g.dinode.itype == InodeType::Empty {
                panic!("inode lock: trying to lock an inode whose type is empty");
            }
        }
        g
    }
}

impl Clone for Inode {
    fn clone(&self) -> Self {
        let mut g = ICACHE.meta.lock();
        g[self.index].refs += 1;
        Self {
            dev: self.dev,
            inum: self.inum,
            index: self.index,
        }
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        ICACHE.put(self);
    }
}
