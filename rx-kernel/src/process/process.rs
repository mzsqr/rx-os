use core::{cell::UnsafeCell, ptr::null_mut};

use crate::{
    arch::riscv::qemu::{fs::NFILE, layout::STACK_SIZE},
    fs::{file::VFile, inode::Inode},
    lock::{Mutex, MutexGuard},
};
use alloc::{boxed::Box, sync::Arc};
use array_macro::array;

use crate::{
    arch::riscv::{
        qemu::layout::{PGSIZE, TRAMPOLINE, TRAPFRAME},
        register::satp,
    },
    memory::{
        address::{PhysicalAddress, VirtualAddress},
        mapping::{pagetable::PageTable, pagetable_entry::PteFlags},
    },
    println,
    process::cpu::cpuid,
};

use super::{
    context::Context, cpu::CPUManager, fork_ret, manager::PROC_MANAGER, trapframe::Trapframe,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcState {
    Unused,
    Used,
    Sleeping,
    Runnable,
    Running,
    Zombie,
    Allocated,
}

pub struct ProcMeta {
    pub state: ProcState,
    pub chan: usize, // If non-zero, sleeping on chan.
    pub killed: bool,
    // TODO: Maybe use Result
    pub xstate: usize, // Exit status to be returned to parent's wait
    pub pid: usize,    // 进程号，和下面的索引号不同
}

impl ProcMeta {
    pub const fn new() -> Self {
        Self {
            state: ProcState::Unused,
            chan: 0,
            killed: false,
            xstate: 0,
            pid: 0,
        }
    }

    pub fn set_state(&mut self, state: ProcState) {
        self.state = state
    }
}

pub struct ProcData {
    // TODO: 用指针表示这个栈是否更加方便
    pub kstack: usize, // 内核栈的虚拟地址
    pub size: usize,   // 进程占用内存大小
    pub pagetable: Option<Box<PageTable>>,
    pub trapframe: *mut Trapframe, // trampoline.S 使用的数据页面
    pub context: Context,          // switch() 用这里保存的数据恢复现场
    pub name: [u8; 16],            // 进程名
    // parent
    pub id: usize, // 用于索引进程表以确定父子关系
    pub open_files: [Option<Arc<VFile>>; NFILE],
    pub cwd: Option<Inode>, // cwd
}

impl ProcData {
    pub const fn new(id: usize) -> Self {
        Self {
            kstack: 0,
            size: 0,
            pagetable: None,
            trapframe: null_mut(),
            context: Context::new(),
            name: [0; 16],
            id,
            open_files: array![_ => None; NFILE],
            cwd: None,
        }
    }

    pub fn get_trapframe(&self) -> *mut Trapframe {
        self.trapframe
    }

    pub fn set_name(&mut self, name: &[u8]) {
        let end = self.name.len().min(name.len());
        self.name[..end].copy_from_slice(&name[..end]);
    }

    // set parent

    pub fn set_kstack(&mut self, kstack: usize) {
        self.kstack = kstack;
    }

    pub fn set_trapframe(&mut self, trapframe: *mut Trapframe) {
        self.trapframe = trapframe
    }

    pub fn set_pagetable(&mut self, pgt: Box<PageTable>) {
        self.pagetable.replace(pgt);
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.context = ctx;
    }

    pub fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn init_context(&mut self) {
        let kstack = self.kstack;
        self.context.write_zero();
        // write forkret
        // child process will return user space from fork ret
        self.context.write_ra(fork_ret as usize);
        self.context.write_sp(kstack + STACK_SIZE);
    }

    /// 为给定进程分配一个页表
    /// 不分配实际内存，会映射Trapoline和Trapframe页面
    ///
    /// # Safety
    /// 要提前分配Trapframe
    pub unsafe fn proc_pagetable(&mut self) -> Option<Box<PageTable>> {
        unsafe extern "C" {
            fn trampoline();
        }

        let mut pgt = PageTable::unew();
        // TODO: chain this error with Option in map function
        if !unsafe {
            pgt.map(
                VirtualAddress::new(TRAMPOLINE),
                PhysicalAddress::new(trampoline as usize),
                PGSIZE,
                PteFlags::R | PteFlags::X,
            )
        } {
            pgt.ufree(0);
            return None;
        }

        if !unsafe {
            pgt.map(
                VirtualAddress::new(TRAPFRAME),
                PhysicalAddress::new(self.trapframe as usize),
                PGSIZE,
                PteFlags::R | PteFlags::W,
            )
        } {
            pgt.ufree(0);
            return None;
        }

        Some(pgt)
    }

    pub fn user_init(&mut self) {
        unsafe extern "C" {
            fn user_trap();
        }

        let tf = unsafe { &mut *self.trapframe };

        tf.kernel_satp = unsafe { satp::read() };
        tf.kernel_sp = self.kstack + STACK_SIZE;
        tf.kernel_trap = user_trap as usize;
        tf.kernel_hartid = unsafe { cpuid() };
    }

    pub fn find_unallocated_fd(&self) -> Result<usize, &'static str> {
        for fd in 0..self.open_files.len() {
            if self.open_files[fd].is_none() {
                return Ok(fd);
            }
        }
        Err("Failed to find unallocated fd")
    }
}

pub struct Process {
    pub meta: Mutex<ProcMeta>,
    pub data: UnsafeCell<ProcData>,
}

/// 保存在data中的数据是在修改meta之后访问的
/// 由于meta中的数据能够保证一致性
/// 所以随后的访问是安全的
unsafe impl Sync for Process {}

impl Process {
    pub const fn new(id: usize, name: &'static str) -> Self {
        Self {
            meta: Mutex::new(ProcMeta::new(), name),
            data: UnsafeCell::new(ProcData::new(id)),
        }
    }

    pub fn init(&self, kstack: usize) {
        let pdata = unsafe { self.data.as_mut_unchecked() };
        pdata.open_files = array![_ => None; NFILE];
        pdata.kstack = kstack;
    }

    pub fn killed(&self) -> bool {
        self.meta.lock().killed
    }

    pub fn pid(&self) -> usize {
        self.meta.lock().pid
    }

    pub fn set_state(&self, state: ProcState) {
        self.meta.lock().set_state(state);
    }

    pub fn set_killed(&self, killed: bool) {
        self.meta.lock().killed = killed;
    }

    pub fn state(&self) -> ProcState {
        self.meta.lock().state
    }

    pub fn name(&self) -> &[u8] {
        unsafe { &self.data.as_ref_unchecked().name }
    }

    pub fn page_table(&mut self) -> &mut PageTable {
        unsafe { self.data.as_mut_unchecked().pagetable.as_mut().unwrap() }
    }

    pub fn proc_pagetable(&self) -> Option<Box<PageTable>> {
        unsafe { self.data.as_mut_unchecked().proc_pagetable() }
    }

    pub fn free_proc(&self) {
        let pdata = unsafe { self.data.as_mut_unchecked() };

        // the trapframe page allocated should be free
        // TODO: we change the trapframe to be owned
        // now let us assume it is leaked
        unsafe {
            let _ = Box::from_raw(pdata.trapframe);
        }

        if let Some(mut pgt) = pdata.pagetable.take() {
            // TODO: this function should be paired in the page table
            // TODO: 将进程大小保存在页表中
            pgt.proc_free_pagetable(pdata.size);
            // this page table is freed automatic
        }
        pdata.size = 0;
        // TODO: parent should be modified

        let mut meta = self.meta.lock();
        meta.pid = 0;
        meta.chan = 0;
        meta.killed = false;
        meta.xstate = 0;
        meta.set_state(ProcState::Unused);

        drop(meta);
    }

    pub fn grow_proc(&self, count: isize) -> Result<(), &'static str> {
        let pdata = unsafe { self.data.as_mut_unchecked() };
        let mut size = pdata.size;
        if let Some(pgt) = pdata.pagetable.as_deref_mut() {
            match count.cmp(&0) {
                core::cmp::Ordering::Less => {
                    let nsz = (size as isize + count) as usize;
                    size = pgt.udealloc(size, nsz);
                }
                core::cmp::Ordering::Equal => {}
                core::cmp::Ordering::Greater => {
                    if let Some(nsz) = unsafe { pgt.ualloc(size, size + count as usize) } {
                        size = nsz;
                    } else {
                        return Err("Fail to allocate virtual memory for user");
                    }
                }
            }
        }
        pdata.size = size;

        Ok(())
    }

    pub fn yielding(&self) {
        let mut pmeta = self.meta.lock();
        pmeta.set_state(ProcState::Runnable);
        unsafe {
            let c = CPUManager::mycpu();
            pmeta = c.sched(pmeta, self.data.as_mut_unchecked().get_context_mut());
        }
        drop(pmeta);
    }

    pub fn sleep<T>(&self, chan: usize, lock: MutexGuard<'_, T>) {
        let mut g = self.meta.lock();
        drop(lock);

        g.chan = chan;
        g.set_state(ProcState::Sleeping);
        unsafe {
            let c = CPUManager::mycpu();
            let ctx = self.data.as_mut_unchecked().get_context_mut();
            g = c.sched(g, ctx);
            g.chan = 0;
            drop(g);
        }
    }

    pub fn fd_alloc(&self, file: &VFile) -> Result<usize, &'static str> {
        let pdata = unsafe { self.data.as_mut_unchecked() };
        let fd = pdata.find_unallocated_fd()?;
        pdata.open_files[fd].replace(Arc::new(file.clone()));
        Ok(fd)
    }

    pub fn fork(&self) -> Option<&Self> {
        if let Some(proc) = PROC_MANAGER.alloc_proc() {
            let pdata = unsafe { self.data.as_mut_unchecked() };
            let cdata = unsafe { proc.data.as_mut_unchecked() };
            if let Some((pgt, ch_pgt)) = pdata
                .pagetable
                .as_deref_mut()
                .zip(cdata.pagetable.as_deref_mut())
            {
                unsafe {
                    pgt.ucopy(ch_pgt, pdata.size)
                        .expect("fork: Failed to copy data from parent process.")
                };
            }

            let ptf = pdata.trapframe as *const _;
            let ch_ptf = cdata.trapframe;
            unsafe {
                *ch_ptf = *ptf;
                (*ch_ptf).ax[0] = 0;
            }

            //  Files
            cdata.open_files.clone_from(&pdata.open_files);
            cdata.cwd.clone_from(&pdata.cwd);

            cdata.name = pdata.name;
            cdata.size = pdata.size;

            let mut child_meta = proc.meta.lock();
            child_meta.state = ProcState::Runnable;
            drop(child_meta);

            let mut wg = PROC_MANAGER.wait_list.lock();
            wg[pdata.id] = self as *const Process as usize;
            drop(wg);

            Some(proc)
        } else {
            println!("[rx-os] fork: No process available");
            None
        }
    }
}
