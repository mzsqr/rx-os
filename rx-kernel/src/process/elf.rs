use core::ptr::slice_from_raw_parts;

use crate::{
    arch::riscv::qemu::{layout::PGSIZE, param::MAXARG},
    fs::{
        inode::{ICACHE, InodeData},
        log::Log,
    },
    lock::SleepMutexGuard,
    memory::{
        address::{Addr, VirtualAddress},
        mapping::{page_round_up, pagetable::PageTable},
    },
};

use super::cpu::CPUManager;

const ELF_MAGIC: u32 = 0x464C457F;

// Values for Proghdr type
const ELF_PROG_LOAD: u32 = 1;

// Flag bits for Proghdr flags
const ELF_PROG_FLAG_EXEC: usize = 1;
const ELF_PROG_FLAG_WRITE: usize = 2;
const ELF_PROG_FLAG_READ: usize = 4;

#[repr(C)]
#[derive(Default)]
pub struct ElfHeader {
    magic: u32,
    elf: [u8; 12],
    etype: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Default)]
pub struct ProgHeader {
    pub prog_type: u32,
    pub flags: u32,
    pub off: usize,
    pub vaddr: usize,
    pub paddr: usize,
    pub file_size: usize,
    pub mem_size: usize,
    pub align: usize,
}

fn load_seg(
    pgt: &mut PageTable,
    va: usize,
    inode_data: &mut SleepMutexGuard<InodeData>,
    offset: usize,
    size: usize,
) -> Result<(), &'static str> {
    let mut va = VirtualAddress::new(va);
    if !va.is_page_aligned() {
        panic!("load seg: va must be page aligned.");
    }

    let mut copy_size = 0;
    while copy_size < size {
        if let Some(pa) = pgt.pgt_translate(va) {
            let count = if size - copy_size < PGSIZE {
                size - copy_size
            } else {
                PGSIZE
            };

            inode_data.read(
                false,
                pa.as_usize(),
                (offset + copy_size) as u32,
                count as u32,
            )?;
        } else {
            panic!("load_seg: Fail to read inode.");
        }

        copy_size += PGSIZE;
        va.add_page();
    }

    Ok(())
}

// TODO: modify argv to contain str
pub unsafe fn exec(path: &str, argv: &[*const u8]) -> Result<usize, &'static str> {
    let mut elf = ElfHeader::default();
    let mut ph = ProgHeader::default();

    Log::begin_op();
    let inode = ICACHE
        .namei(path.as_bytes())
        .ok_or("Fail to find executable file")?;

    let mut ig = inode.lock();

    // 读取Elf头
    // FIXME: 不能用map_err，会导致ig被move到闭包中
    if ig
        .read(
            false,
            &elf as *const _ as usize,
            0,
            size_of::<ElfHeader>() as u32,
        )
        .is_err()
    {
        drop(ig);
        Log::end_op();
        return Err("exec: Failed to read elf header.");
    };

    if elf.magic != ELF_MAGIC {
        drop(ig);
        Log::end_op();
        return Err("exec: Elf magic number is wrong");
    }

    let proc = unsafe { CPUManager::myproc().unwrap() };
    // 为进程分配一个新的进程表
    //  旧的页表仍然为进程所有
    // 进程释放页面后take即可销毁
    let mut pgt = proc
        .proc_pagetable()
        .expect("exec: Fail to alloc pagetable for current process.");

    let ph_size = size_of::<ProgHeader>() as u32;
    let mut off = elf.phoff;
    let mut size = 0;
    for _ in 0..elf.phnum {
        if ig
            .read(false, &ph as *const _ as usize, off as u32, ph_size)
            .is_err()
        {
            // page table is freed automatic
            drop(ig);
            Log::end_op();
            return Err("exec: Fail to read from inode");
        };

        // FIXME: continue but do not add off
        if ph.prog_type != ELF_PROG_LOAD {
            continue;
        }
        if ph.mem_size < ph.file_size {
            pgt.proc_free_pagetable(size);
            drop(ig);
            Log::end_op();
            return Err("exec: memory size is less than file size.");
        }

        if ph.vaddr.checked_add(ph.mem_size).is_none() {
            pgt.proc_free_pagetable(size);
            drop(ig);
            Log::end_op();
            return Err("exec: vaddr + memsize < vaddr");
        }

        if let Some(sz) = unsafe { pgt.ualloc(size, ph.vaddr + ph.mem_size) } {
            size = sz;
        } else {
            pgt.proc_free_pagetable(size);
            drop(ig);
            Log::end_op();
            return Err("exec: Fail to ualloc");
        }

        if ph.vaddr % PGSIZE != 0 {
            pgt.proc_free_pagetable(size);
            drop(ig);
            Log::end_op();
            return Err("Exec: Program header must be integer multiple of PGSIZE.");
        }

        if load_seg(pgt.as_mut(), ph.vaddr, &mut ig, ph.off, ph.file_size).is_err() {
            pgt.proc_free_pagetable(size);
            drop(ig);
            Log::end_op();
            return Err("exec: Fail to load segment");
        }

        off += size_of::<ProgHeader>() as u64;
    }

    drop(ig);
    Log::end_op();

    size = page_round_up(size);
    if let Some(sz) = unsafe { pgt.ualloc(size, size + 2 * PGSIZE) } {
        size = sz;
    } else {
        pgt.proc_free_pagetable(size);
        return Err("exec: Fail to ualloc");
    }

    pgt.uclear(VirtualAddress::new(size - 2 * PGSIZE)); // 用户栈保护页
    let mut sp = size;
    let stack_base = sp - PGSIZE;

    let mut argc = 0;
    let mut user_stack = [0_usize; MAXARG];
    loop {
        if argv[argc] as usize == 0 {
            break;
        }
        if argc >= MAXARG {
            pgt.proc_free_pagetable(size);
            return Err("exec: argc is more than MAXARG");
        }
        let strlen = unsafe {
            let mut st = argv[argc];
            while *st != b'0' {
                st = st.add(1);
            }
            st as usize - argv[argc] as usize
        };
        sp -= strlen + 1;
        sp = align_sp(sp);
        if sp < stack_base {
            pgt.proc_free_pagetable(size);
            return Err("exec: User stack Bomb!");
        }

        let src = unsafe { &*slice_from_raw_parts(argv[argc], strlen + 1) };
        if pgt.copy_out(sp, src).is_err() {
            pgt.proc_free_pagetable(size);
            return Err("exec: Fail to copy out");
        }
        user_stack[argc] = sp;
        argc += 1;
    }
    user_stack[argc] = 0;

    sp -= (argc + 1) * size_of::<usize>();
    sp = align_sp(sp);
    if sp < stack_base {
        // Log::end_op();
        pgt.proc_free_pagetable(size);
    }

    pgt.copy_out(sp, unsafe {
        &*slice_from_raw_parts(
            user_stack.as_ptr() as *const u8,
            user_stack.len() * size_of::<usize>(),
        )
    })
    .map_err(|_| {
        pgt.proc_free_pagetable(size);
        "Exec: Fail to copy out."
    })?;

    let pdata = unsafe { proc.data.as_mut_unchecked() };
    let tf = unsafe { &mut *pdata.trapframe };
    tf.ax[1] = sp;

    for (idx, b) in path.bytes().enumerate() {
        if idx >= pdata.name.len() {
            break;
        }
        pdata.name[idx] = b;
    }

    // install new pagetable and delete old one
    if let Some(mut old_pgt) = pdata.pagetable.replace(pgt) {
        old_pgt.proc_free_pagetable(pdata.size);
        // delete old pagetable
    }

    pdata.size = size;
    tf.epc = elf.entry as usize;
    tf.sp = sp;

    Ok(size)
}

#[inline]
fn align_sp(sp: usize) -> usize {
    sp - (sp % 16)
}
