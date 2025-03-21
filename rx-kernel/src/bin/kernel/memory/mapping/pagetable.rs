//! 页表的抽象
//!
//! The risc-v Sv39 scheme has three levels of page-table
//! pages. A page-table page contains 512 64-bit PTEs.
//! A 64-bit virtual address is split into five fields:
//!   39..63 -- must be zero.
//!   30..38 -- 9 bits of level-2 index.
//!   21..29 -- 9 bits of level-1 index.
//!   12..20 -- 9 bits of level-0 index.
//!    0..11 -- 12 bits of byte offset within the page.
//!
//! Look up a virtual address, return the physical address,
//! or 0 if not mapped.
//! Can only be used to look up user pages.
//!
//! 启用巨页时，虚拟地址的0..20都用于索引页内部的偏移量
//! 这时需要在1级页表的页表项中设置所需的RWX标志位
//!
//! 进程地址空间视图：
//!     0.. text segment RXU
//!     ..  data segment RWU
//!     ..  stack segment (4 pages default) RWU
//!     ..  heap segment RWU
//!     ..  trapframe (1 page) RW
//!     ..  MAXVA trap(trampoline) RX

use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use alloc::boxed::Box;

use crate::{
    arch::riscv::qemu::layout::{
        MAXVA, PGSHIFT, PGSIZE, TRAMPOLINE, TRAPFRAME, USTACK_BASE, USTACK_SIZE,
    },
    memory::{
        PageAllocator, RawPage, UStack,
        address::{Addr, PhysicalAddress, VirtualAddress},
    },
    println,
};

use super::{
    page_round_up,
    pagetable_entry::{PageTableEntry, PteFlags},
};

#[derive(Debug, Clone)]
#[repr(C, align(0x1000))]
pub struct PageTable {
    pub entries: [PageTableEntry; PGSIZE / 8],
}

/// 这是树形的页表
/// 它所有的内容是它所支配的页表
/// TODO: 页表也许能够拥有它所管理的页面，这样进一步实现完全的所有权
impl PageTable {
    /// 页表的物理地址（数组起始地址）
    pub fn as_addr(&self) -> usize {
        self.entries.as_ptr() as usize
    }

    pub const fn empty() -> Self {
        Self {
            entries: [PageTableEntry(0); PGSIZE / 8],
        }
    }

    // pub fn look(&mut self) {
    //     let va = STACK0.as_ptr() as usize;
    //     let e = self.translate(VirtualAddress::new(va), false).unwrap();
    //     println!("{va:#x} {:#x}", e.as_pagetable() as usize);
    // }

    pub fn debug(&self, _level: i32, virt: usize) {
        self.entries.iter().enumerate().for_each(|(idx, e)| {
            let va = (virt << 9) + (idx << 12);
            if e.is_valid() && e.is_leaf() {
                // for i in level..=3 {
                //     print!(".");
                // }

                println!(
                    "pte: {:#X} virtual: {:#X}, physical: {:#X}, flags: {:#b}",
                    e.as_usize(),
                    va,
                    e.as_pagetable() as usize,
                    e.as_flags()
                );
                // if va != TRAMPOLINE {
                //     assert_eq!(va, e.as_pagetable() as usize);
                // }
            }
            if e.is_valid() && !e.is_leaf() {
                unsafe {
                    let child_pgt = &mut *(e.as_pagetable());
                    child_pgt.debug(_level - 1, va);
                }
            }
        });
    }

    /// 将当前页表地址转为satp寄存器接受的页表地址
    pub fn as_satp(&self) -> usize {
        crate::arch::riscv::register::satp::SATP_SV39
            | ((self.entries.as_ptr() as usize) >> PGSHIFT)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entries.iter_mut().for_each(|x| x.write_zero());
    }

    pub fn write(&mut self, page_table: &PageTable) {
        self.entries
            .iter_mut()
            .zip(&page_table.entries)
            .for_each(|(dst, src)| dst.write(src.as_usize()));
    }

    /// 递归地删除页表
    ///
    /// # Safety
    /// 必须保证在此之前将该页表映射的所有页面全部回收
    #[deprecated]
    pub unsafe fn free(&mut self) {
        self.entries.iter_mut().for_each(|e| {
            if e.is_valid() && !e.is_leaf() {
                unsafe {
                    let child_pgt = &mut *(e.as_pagetable());
                    child_pgt.free();
                    let _ = Box::from_raw(child_pgt as *mut Self);
                }
            } else if e.is_valid() {
                panic!("pagetable free(): leaf not be removed");
            }
        });
        // unsafe {
        //     let _ = Box::from_raw(self as *mut Self);
        // }
    }

    // TODO: 当采用巨页时需要修改此处，巨页的页表项的标志位会带有R/W/X
    /// 将虚拟地址翻译为物理地址，返回页表项
    /// 将alloc设置为true会同时分配相关的页表
    pub fn translate(&mut self, va: VirtualAddress, alloc: bool) -> Option<&mut PageTableEntry> {
        if va.as_usize() > MAXVA {
            return None;
        }

        let mut pgt = self;
        for level in (1..=2).rev() {
            let pte = &mut pgt.entries[va.page_num(level)];
            if pte.is_valid() {
                // 这里是安全的，因为已经确认了这个页表项指向已分配的有效物理地址
                pgt = unsafe { &mut *(pte.as_pagetable()) };
            } else {
                if !alloc {
                    return None;
                }
                // 这里分配一个页面
                let zeroed_pgt = unsafe { Box::<PageTable>::new_zeroed().assume_init() };
                pte.write_perm(PhysicalAddress(zeroed_pgt.as_addr()), PteFlags::empty());
                // 之后需要手动管理这个页面
                pgt = Box::leak(zeroed_pgt);
            }
        }
        Some(&mut pgt.entries[va.page_num(0)])
    }

    /// 在给定页表上将虚拟地址转为物理地址
    /// 仅用于用户页面物理地址的查找
    pub fn pgt_translate(&mut self, va: VirtualAddress) -> Option<PhysicalAddress> {
        let addr = va.as_usize();
        if addr > MAXVA {
            return None;
        }

        if let Some(pte) = self.translate(va, false) {
            if !pte.is_valid() | !pte.is_user() {
                return None;
            }

            Some(PhysicalAddress(pte.as_pagetable() as usize))
        } else {
            None
        }
    }

    /// # Safety
    /// 请保证映射的物理地址是有效的地址
    /// 将虚拟页面映射到物理页面
    pub unsafe fn map(
        &mut self,
        mut va: VirtualAddress,
        mut pa: PhysicalAddress,
        size: usize,
        perm: PteFlags,
    ) -> bool {
        let mut last = VirtualAddress::new(va.as_usize() + size);
        va.pg_round_down();
        last.pg_round_up();
        while va != last {
            if let Some(pte) = self.translate(va, true) {
                if pte.is_valid() {
                    println!(
                        "va: {:#x}, pa: {:#x}, pte: {:#x}",
                        va.as_usize(),
                        pa.as_usize(),
                        pte.0
                    );
                    panic!("remap");
                }

                pte.write_perm(pa, perm);
                va.add_page();
                pa.add_page();
            } else {
                return false;
            }
        }

        true
    }

    /// 为内核页表添加映射
    ///
    /// # Safety
    /// 保证映射的物理地址是有效的
    /// 只在系统启动时进行映射
    /// 映射时不能启用TLB或者分页
    /// # Panics
    /// 一旦映射失败会引起panic
    pub unsafe fn kernel_map(
        &mut self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        size: usize,
        perm: PteFlags,
    ) {
        // println!("{:#x} {:#x}", va.as_usize(), pa.as_usize());
        if !unsafe { self.map(va, pa, size, perm) } {
            panic!("内核虚拟地址映射失败");
        }
    }

    /// 创建空的用户进程页表
    /// 当内存不足时
    /// TODO: 内存不足时页表分配
    pub fn unew() -> Box<PageTable> {
        unsafe { Box::new_zeroed().assume_init() }
    }

    /// 为第一个进程加载程序
    ///
    /// # Safety
    /// 加载的程序应当小于一个页面
    pub unsafe fn uinit(&mut self, src: &[u8]) {
        if src.len() >= PGSIZE {
            panic!("uinit: more than a page");
        }

        let mem = unsafe { RawPage::new_zeroed() };
        println!("First Addr: {:#x}", mem as *mut RawPage as usize);
        mem.data.fill(0);

        unsafe {
            self.map(
                VirtualAddress::new(0),
                PhysicalAddress::new(mem as *const RawPage as usize),
                PGSIZE,
                PteFlags::R | PteFlags::W | PteFlags::X | PteFlags::U,
            );
        }

        self.ualloc_stack();

        mem.data[..src.len()].copy_from_slice(src);
    }

    /// 为用户分配页表项以及物理内存页面
    /// 使得用户空间从old增长到new
    /// 地址无需提前对齐
    ///
    /// # Safety
    /// TODO:
    pub unsafe fn ualloc(&mut self, mut old_size: usize, new_size: usize) -> Option<usize> {
        if new_size < old_size {
            return Some(old_size);
        }

        old_size = page_round_up(old_size);
        for cur_size in (old_size..new_size).step_by(PGSIZE) {
            let mem = unsafe { RawPage::new_zeroed() };
            let addr = mem as *const RawPage as usize;
            mem.data.fill(0);

            unsafe {
                if !self.map(
                    VirtualAddress::new(cur_size),
                    PhysicalAddress::new(addr),
                    PGSIZE,
                    // FIXME: custom perm
                    PteFlags::W | PteFlags::R | PteFlags::X | PteFlags::U,
                ) {
                    mem.drop();
                    self.udealloc(cur_size, old_size);
                    return None;
                }
            }
        }

        Some(new_size)
    }

    pub fn ualloc_stack(&mut self) -> bool {
        let mem = unsafe { UStack::new_zeroed() };
        let addr = mem as *const UStack as usize;
        mem.data.fill(0);

        unsafe {
            if !self.map(
                VirtualAddress::new(USTACK_BASE),
                PhysicalAddress::new(addr),
                USTACK_SIZE,
                // FIXME: custom perm
                PteFlags::W | PteFlags::R | PteFlags::U,
            ) {
                mem.drop();
                return false;
            }
        }

        let guard = unsafe { RawPage::new_zeroed() };
        let addr = guard as *const RawPage as usize;

        unsafe {
            if !self.map(
                VirtualAddress::new(USTACK_BASE - PGSIZE),
                PhysicalAddress::new(addr),
                PGSIZE,
                PteFlags::empty(),
            ) {
                self.uunmap(VirtualAddress::new(USTACK_BASE), USTACK_SIZE / PGSIZE, true);
                guard.drop();
                return false;
            }
        }

        true
    }

    pub fn ufree_stack(&mut self) {
        self.uunmap(
            VirtualAddress(USTACK_BASE - PGSIZE),
            USTACK_SIZE / PGSIZE + 1,
            true,
        );
    }

    pub fn ucopy_stack(&mut self, other: &mut Self) {
        unsafe {
            // TODO: DON'T COPY GUARD
            self.ucopy(
                other,
                VirtualAddress::new(USTACK_BASE - PGSIZE),
                USTACK_SIZE + PGSIZE,
            )
            .unwrap()
        }
    }

    /// 释放用户内存页面
    /// 释放用户页表
    pub fn ufree(&mut self, size: usize) {
        if size > 0 {
            let ppn = page_round_up(size) / PGSIZE;
            self.uunmap(VirtualAddress::new(0), ppn, true);
        }
    }

    /// 缩小用户进程空间大小
    /// 大小无需提前对齐
    /// 如果新的大小大于旧的大小则不会有动作
    pub fn udealloc(&mut self, old_size: usize, new_size: usize) -> usize {
        if new_size >= old_size {
            return old_size;
        }

        if page_round_up(new_size) < page_round_up(old_size) {
            let npages = (page_round_up(old_size) - page_round_up(new_size)) / PGSIZE;
            self.uunmap(VirtualAddress::new(page_round_up(new_size)), npages, true);
        }

        new_size
    }

    /// 在分配给用户的页面中连续删除npages个页面
    /// 要求地址是对其的
    ///
    /// # Panics
    /// 当虚拟地址非页对齐会panic
    pub fn uunmap(&mut self, mut va: VirtualAddress, npages: usize, free: bool) {
        if !va.is_page_aligned() {
            panic!("uunmap: not aligned.");
        }

        for _ in 0..npages {
            if let Some(pte) = self.translate(va, false) {
                if !pte.is_valid() {
                    panic!("uunmap: not mapped");
                }
                if !pte.as_flags() == PteFlags::V.bits() {
                    panic!("uunmap: not a leaf");
                }

                if free {
                    // TODO: 需要一个更加清晰的销毁方法
                    let pa = pte.as_pagetable() as *mut RawPage;
                    unsafe {
                        let _ = Box::from_raw(pa);
                    }
                    pte.write(0);
                }
            } else {
                panic!("uunmap");
            }

            va.add_page();
        }
    }

    /// 根据父进程的页表信息将父进程的内存拷贝给子进程并映射
    /// 失败时会自动释放内存
    /// self --> other
    ///
    /// # Safety
    /// TODO:
    pub unsafe fn ucopy(
        &mut self,
        other: &mut Self,
        mut va: VirtualAddress,
        size: usize,
    ) -> Result<(), &'static str> {
        // let mut va = VirtualAddress::new(0);
        let start = va.as_usize();
        while va.as_usize() != size + start {
            if let Some(pte) = self.translate(va, false) {
                if !pte.is_valid() {
                    panic!("ucopy: page not present");
                }

                let pgt = pte.as_pagetable();
                let flags = pte.as_flags();
                let flags = PteFlags::new(flags);

                let mut alloc_pgt = Box::new(PageTable::empty());
                alloc_pgt
                    .entries
                    .copy_from_slice(unsafe { &(*pgt).entries });

                if !unsafe {
                    other.map(va, PhysicalAddress::new(alloc_pgt.as_addr()), PGSIZE, flags)
                } {
                    // alloc_pgt will be deallocate because we do not leak it
                    other.uunmap(VirtualAddress::new(0), va.as_usize() / PGSIZE, true);
                    return Err("ucopy: Failed.");
                } else {
                    Box::leak(alloc_pgt);
                }
            } else {
                panic!("ucopy: No exist pte(pte should exist)");
            }

            va.add_page();
        }

        Ok(())
    }

    /// 清除va对应虚拟地址的表项中的User位
    /// 使得用户无法访问该页面
    /// 用于用户栈的保护页面
    pub fn uclear(&mut self, va: VirtualAddress) {
        if let Some(pte) = self.translate(va, false) {
            pte.rm_user_bit();
        } else {
            panic!("uclear: Not found valid pte for virtual address");
        }
    }

    /// 从内核空间将src指向的内存区域复制到用户空间中。
    /// 拷贝整个src到用户空间。
    ///
    /// 安全性的考虑请参见`copy_in`
    pub fn copy_out(&mut self, dst: usize, mut src: &[u8]) -> Result<(), &'static str> {
        let mut va = VirtualAddress::new(dst);
        va.pg_round_down();

        let mut count = PGSIZE - (dst - va.as_usize());
        let mut pa = self.pgt_translate(va).unwrap();
        let mut dst = &mut unsafe { &mut *slice_from_raw_parts_mut(pa.as_mut_ptr(), PGSIZE) }
            [dst - va.as_usize()..];

        loop {
            if count > src.len() {
                count = src.len();
            }
            dst[..count].copy_from_slice(&src[..count]);
            src = &src[count..];
            if src.is_empty() {
                break;
            }
            va.add_page();
            pa = self.pgt_translate(va).unwrap();
            count = PGSIZE;
            dst = unsafe { &mut *slice_from_raw_parts_mut(pa.as_mut_ptr(), PGSIZE) };
        }

        Ok(())
    }

    /// 将用户空间由src指向的`dst.len()`字节的内存空间复制到内核空间中。
    /// 用户空间的物理内存不一定连续，所以需要对跨页的内存镜像处理。
    ///
    /// 进程访问的内存如果超过了其地址空间自然不能被页表翻译，所以调用这个函数对内核来说是安全的。
    pub fn copy_in(&mut self, mut dst: &mut [u8], src: usize) -> Result<(), &'static str> {
        let mut va = VirtualAddress::new(src);
        va.pg_round_down();

        let mut count = PGSIZE - (src - va.as_usize());
        let mut pa = self.pgt_translate(va).unwrap();
        let mut src =
            &unsafe { &*slice_from_raw_parts(pa.as_mut_ptr(), PGSIZE) }[src - va.as_usize()..];

        loop {
            if count > dst.len() {
                count = dst.len();
            }
            dst[..count].copy_from_slice(&src[..count]);
            dst = &mut dst[count..];
            if dst.is_empty() {
                break;
            }
            va.add_page();
            pa = self.pgt_translate(va).unwrap();
            count = PGSIZE;
            src = unsafe { &mut *slice_from_raw_parts_mut(pa.as_mut_ptr(), PGSIZE) };
        }

        Ok(())
    }

    /// 处理src内容读取过程中可能为0的情况
    /// 最多读取dst能存的最大值（dst.len())
    pub fn copy_in_str(&mut self, mut dst: &mut [u8], src: usize) -> Result<(), &'static str> {
        let mut va = VirtualAddress::new(src);
        va.pg_round_down();

        let mut count = PGSIZE - (src - va.as_usize());
        let mut pa = self.pgt_translate(va).unwrap();
        let mut src =
            &unsafe { &*slice_from_raw_parts(pa.as_mut_ptr(), PGSIZE) }[src - va.as_usize()..];

        loop {
            if count > dst.len() {
                count = dst.len();
            }
            if let Some((end_idx, _)) = src.iter().enumerate().find(|(_, x)| **x == 0) {
                src = &src[..end_idx + 1];
                count = end_idx + 1;
                dst = &mut dst[..count]; // it will be empty after copy 
            }
            dst[..count].copy_from_slice(src);
            dst = &mut dst[count..];
            if dst.is_empty() {
                break;
            }
            va.add_page();
            pa = self.pgt_translate(va).unwrap();
            count = PGSIZE;
            src = unsafe { &mut *slice_from_raw_parts_mut(pa.as_mut_ptr(), PGSIZE) };
        }

        Ok(())
    }

    pub fn proc_free_pagetable(&mut self, size: usize) {
        // TODO: 或者这个放在进程表中
        self.uunmap(VirtualAddress::new(TRAMPOLINE), 1, false);
        self.uunmap(VirtualAddress::new(TRAPFRAME), 1, false);
        self.ufree(size);
    }
}

/// 根页表会负责清除名下所有泄漏的内存
impl Drop for PageTable {
    /// 在销毁前一定要清除所有的物理页面映射
    fn drop(&mut self) {
        self.entries.iter_mut().for_each(|e| {
            if e.is_valid() && !e.is_leaf() {
                unsafe {
                    let child_pgt = &mut *(e.as_pagetable());
                    // 然后又进一步销毁它名下的所有表
                    let _ = Box::from_raw(child_pgt as *mut Self);
                }
            } else if e.is_valid() {
                // FIXME: free from kernel space has bug
                // panic!("pagetable free(): leaf not be removed");
            }
        });
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use crate::{
        arch::riscv::qemu::layout::PGSIZE,
        memory::{
            PageAllocator, RawPage,
            address::{Addr, PhysicalAddress, VirtualAddress},
            kalloc::ALLOCATOR,
            mapping::pagetable_entry::{PTE_W, PteFlags},
        },
        println,
    };

    use super::PageTable;

    #[test_case]
    fn pgt_has_same_addr() {
        let zerod_pgt = unsafe { Box::<PageTable>::new_zeroed().assume_init() };
        let addr_from_arr = zerod_pgt.as_addr();
        let addr = Box::as_ptr(&zerod_pgt) as usize;
        assert_eq!(addr, addr_from_arr, "数组首地址和页表地址不同");
    }

    #[test_case]
    fn pgt_free() {
        let prev = ALLOCATOR.lock().used();
        let mut pgt = PageTable::unew();
        let apage = unsafe { RawPage::new_zeroed() };
        unsafe {
            pgt.map(
                VirtualAddress::new(0),
                PhysicalAddress::new(apage as *mut RawPage as usize),
                PGSIZE,
                PteFlags::U | PteFlags::R | PteFlags::W | PteFlags::X,
            )
        };
        pgt.ufree(PGSIZE);
        // unsafe { pgt.free() };
        drop(pgt);
        let after = ALLOCATOR.lock().used();
        assert_eq!(prev, after, "you do not clear all page table");
    }

    #[test_case]
    fn pgt_copy_out() {
        let mut pgt = PageTable::unew();
        let p1 = unsafe { RawPage::new_zeroed() };
        let p2 = unsafe { RawPage::new_zeroed() };
        let mut addr = 0;
        for apage in [p1, p2] {
            unsafe {
                pgt.map(
                    VirtualAddress::new(addr),
                    PhysicalAddress::new(apage as *mut RawPage as usize),
                    PGSIZE,
                    PteFlags::U | PteFlags::R | PteFlags::W,
                )
            };
            addr += PGSIZE;
        }
        let prev1 = pgt
            .translate(VirtualAddress::new(0), false)
            .unwrap()
            .as_pagetable() as *mut RawPage;
        let prev2 = pgt
            .translate(VirtualAddress::new(PGSIZE), false)
            .unwrap()
            .as_pagetable() as *mut RawPage;
        unsafe {
            (*prev1).data[0] = 222;
            (*prev2).data[0] = 255;
        }
        let data = [111_u8; PGSIZE * 2];
        pgt.copy_out(0, &data);
        assert_eq!(unsafe { (*prev1).data[0] }, 111);
        assert_eq!(unsafe { (*prev2).data[0] }, 111);

        pgt.ufree(PGSIZE * 2);
    }

    #[test_case]
    fn pgt_copy_in() {
        let mut pgt = PageTable::unew();
        let p1 = unsafe { RawPage::new_zeroed() };
        let p2 = unsafe { RawPage::new_zeroed() };
        let mut addr = 0;
        for apage in [p1, p2] {
            unsafe {
                pgt.map(
                    VirtualAddress::new(addr),
                    PhysicalAddress::new(apage as *mut RawPage as usize),
                    PGSIZE,
                    PteFlags::U | PteFlags::R | PteFlags::W,
                )
            };
            addr += PGSIZE;
        }
        let prev1 = pgt
            .translate(VirtualAddress::new(0), false)
            .unwrap()
            .as_pagetable() as *mut RawPage;
        let prev2 = pgt
            .translate(VirtualAddress::new(PGSIZE), false)
            .unwrap()
            .as_pagetable() as *mut RawPage;
        unsafe {
            (*prev1).data[0] = 222;
            (*prev2).data[0] = 255;
        }
        let mut data = [111_u8; PGSIZE * 2];
        pgt.copy_in(&mut data, 0);
        assert_eq!(data[0], 222);
        assert_eq!(data[PGSIZE], 255);

        pgt.ufree(PGSIZE * 2);
    }

    #[test_case]
    fn same_mapping() {
        let mut pgt = PageTable::unew();
        let p1 = unsafe { RawPage::new_zeroed() };
        let addr = p1 as *mut RawPage as usize;
        unsafe {
            pgt.kernel_map(
                VirtualAddress::new(addr),
                PhysicalAddress::new(addr),
                PGSIZE,
                PteFlags::W | PteFlags::R | PteFlags::U,
            );
        }
        assert_eq!(
            addr,
            pgt.pgt_translate(VirtualAddress::new(addr))
                .unwrap()
                .as_usize()
        );
        pgt.uunmap(VirtualAddress::new(addr), 1, true);
    }
}
